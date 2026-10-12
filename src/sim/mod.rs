pub mod channel;

use crate::audio::{ANALYSIS_FS, filter::bandpass_zero_phase};
use crate::geo::LatLon;
use crate::model::params::Atmosphere;
use crate::strikes::Strike;
use anyhow::Result;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Normal};
use std::path::Path;

pub struct Scenario {
    pub listener: LatLon,
    pub start: f64,
    pub duration_s: f64,
    pub atm: Atmosphere,
    /// Rain/wind floor RMS relative to the loudest thunder peak.
    pub noise_level: f64,
    pub car_passes: usize,
    pub seed: u64,
}

const MAX_RANGE_KM: f64 = 60.0;

/// Synthesises thunder audio for `sc` from real `strikes`, jittering each strike by its error ellipse.
pub fn synthesize(strikes: &[Strike], sc: &Scenario) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(sc.seed);
    let n = (sc.duration_s * ANALYSIS_FS) as usize;
    let mut out = vec![0.0f32; n];
    let unit = Normal::new(0.0, 1.0).unwrap();
    for s in strikes {
        if s.time > sc.start + sc.duration_s || s.time < sc.start - MAX_RANGE_KM / 0.3 {
            continue;
        }
        let (e, nn) = sc.listener.enu_km(&s.pos);
        let (je, jn) = jitter(s, &unit, &mut rng);
        let center = (e + je, nn + jn);
        if center.0.hypot(center.1) > MAX_RANGE_KM {
            continue;
        }
        let segs = if s.cloud { channel::in_cloud(center, &mut rng) } else { channel::cloud_to_ground(center, &mut rng) };
        let strength = (s.peak_ka.abs() as f64).max(1.0).sqrt() * if s.cloud { 0.4 } else { 1.0 };
        let wavelet_hz = rng.gen_range(40.0..120.0);
        for seg in &segs {
            emit(&mut out, seg, s.time - sc.start, strength, wavelet_hz, &sc.atm);
        }
    }
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-9);
    out.iter_mut().for_each(|v| *v /= peak);
    add_noise(&mut out, sc, &mut rng);
    out
}

/// Adds one segment's pressure pulse (Ricker wavelet) at its arrival time.
fn emit(out: &mut [f32], seg: &channel::Segment, t_rel: f64, strength: f64, f0: f64, atm: &Atmosphere) {
    // Segments are listener-centred; vector from segment to listener is -pos.
    let r = (seg.pos[0].powi(2) + seg.pos[1].powi(2) + seg.pos[2].powi(2)).sqrt().max(0.05);
    let h = seg.pos[0].hypot(seg.pos[1]).max(1e-6);
    let (ue, un) = (-seg.pos[0] / h, -seg.pos[1] / h);
    let c_kms = (atm.c_mps + atm.wind_e_mps * ue + atm.wind_n_mps * un) / 1000.0;
    let t = t_rel + r / c_kms;
    let cos_los = -(seg.pos[0] * seg.dir[0] + seg.pos[1] * seg.dir[1] + seg.pos[2] * seg.dir[2]) / r;
    let directivity = (1.0 - cos_los * cos_los).max(0.0);
    let shadow = (-(h / 16.0).powi(3)).exp();
    let amp = strength * seg.len_km * directivity * shadow / r;
    let i0 = (t * ANALYSIS_FS) as isize;
    let half = (1.5 * ANALYSIS_FS / f0) as isize;
    for k in -half..=half {
        let i = i0 + k;
        if i < 0 || i as usize >= out.len() {
            continue;
        }
        let tau = std::f64::consts::PI * f0 * k as f64 / ANALYSIS_FS;
        out[i as usize] += (amp * (1.0 - 2.0 * tau * tau) * (-tau * tau).exp()) as f32;
    }
}

fn jitter(s: &Strike, unit: &Normal<f64>, rng: &mut impl Rng) -> (f64, f64) {
    let th = (s.ellipse_angle_deg as f64).to_radians();
    let a = s.semi_major_km as f64 / 1.1774 * unit.sample(rng);
    let b = s.semi_minor_km as f64 / 1.1774 * unit.sample(rng);
    (a * th.sin() + b * th.cos(), a * th.cos() - b * th.sin())
}

/// Slowly varying band-limited rain floor plus passing-car bursts.
fn add_noise(out: &mut [f32], sc: &Scenario, rng: &mut impl Rng) {
    let unit = Normal::new(0.0, 1.0).unwrap();
    let mut noise: Vec<f32> = (0..out.len()).map(|_| unit.sample(rng) as f32).collect();
    bandpass_zero_phase(&mut noise, 20.0, 400.0, ANALYSIS_FS);
    let rms = (noise.iter().map(|v| v * v).sum::<f32>() / noise.len() as f32).sqrt().max(1e-9);
    let mut level = 1.0f64;
    let cars: Vec<f64> = (0..sc.car_passes).map(|_| rng.gen_range(0.0..sc.duration_s)).collect();
    for (i, v) in out.iter_mut().enumerate() {
        if i % 200 == 0 {
            level = (level + 0.05 * unit.sample(rng)).clamp(0.5, 1.5);
        }
        let t = i as f64 / ANALYSIS_FS;
        let car: f64 = cars.iter().map(|c| 3.0 * (-0.5 * ((t - c) / 3.0).powi(2)).exp()).sum();
        *v += (sc.noise_level * (level + car)) as f32 * noise[i] / rms;
    }
}

pub fn write_wav(path: &Path, samples: &[f32]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: ANALYSIS_FS as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &s in samples {
        w.write_sample(s)?;
    }
    w.finalize()?;
    Ok(())
}
