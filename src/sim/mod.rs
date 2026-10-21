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
    /// Two-mic recorder (stereo output); None writes mono.
    pub mics: Option<Mics>,
    /// CG segments below this height (km) radiate weaker, ∝ (z/h)²; 0 = uniform channel.
    pub cg_quiet_below_km: f64,
}

#[derive(Clone, Copy)]
pub struct Mics {
    pub spacing_m: f64,
    /// Compass bearing the left mic sits toward, from the right mic.
    pub axis_deg: f64,
}

/// Fraction of the rain floor that differs between the two mics.
const NOISE_DECORRELATION: f32 = 0.15;

const MAX_RANGE_KM: f64 = 60.0;
const SHADOW_EDGE_KM: f64 = 0.25;

/// Synthesises thunder audio for `sc` from real `strikes`, jittering each strike by its error ellipse.
/// Returns one channel, or [left, right] when `sc.mics` is set.
pub fn synthesize(strikes: &[Strike], sc: &Scenario) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(sc.seed);
    let n = (sc.duration_s * ANALYSIS_FS) as usize;
    let mut out = vec![vec![0.0f32; n]; if sc.mics.is_some() { 2 } else { 1 }];
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
            let w = if s.cloud || sc.cg_quiet_below_km <= 0.0 { 1.0 } else { (seg.pos[2] / sc.cg_quiet_below_km).clamp(0.0, 1.0).powi(2) };
            emit(&mut out, seg, s.time - sc.start, strength * w, wavelet_hz, &sc.atm, sc.mics);
        }
    }
    let peak = out.iter().flatten().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-9);
    out.iter_mut().flatten().for_each(|v| *v /= peak);
    add_noise(&mut out, sc, &mut rng);
    out
}

/// Adds one segment's pressure pulse (Ricker wavelet) at its arrival time on each mic.
fn emit(out: &mut [Vec<f32>], seg: &channel::Segment, t_rel: f64, strength: f64, f0: f64, atm: &Atmosphere, mics: Option<Mics>) {
    // Segments are listener-centred; vector from segment to listener is -pos.
    let r = (seg.pos[0].powi(2) + seg.pos[1].powi(2) + seg.pos[2].powi(2)).sqrt().max(0.05);
    let h = seg.pos[0].hypot(seg.pos[1]).max(1e-6);
    let (ue, un) = (-seg.pos[0] / h, -seg.pos[1] / h);
    let c_kms = (atm.c_mps + atm.wind_e_mps * ue + atm.wind_n_mps * un) / 1000.0;
    let t = t_rel + r / c_kms;
    let cos_los = -(seg.pos[0] * seg.dir[0] + seg.pos[1] * seg.dir[1] + seg.pos[2] * seg.dir[2]) / r;
    let directivity = (1.0 - cos_los * cos_los).max(0.0);
    let shadow = if atm.lapse_k_per_km > 0.0 {
        // Below the grazing ray only diffracted/scattered sound arrives; decays with depth into the shadow.
        (-(atm.shadow_alt_km(h) - seg.pos[2]).max(0.0) / SHADOW_EDGE_KM).exp()
    } else {
        (-(h / 16.0).powi(3)).exp()
    };
    let amp = strength * seg.len_km * directivity * shadow / r;
    // Plane wave from bearing az: the mic displaced toward az hears it (spacing/2)·cos(az−axis)/c earlier.
    let half_lead = mics.map_or(0.0, |m| {
        let az = seg.pos[0].atan2(seg.pos[1]);
        0.5 * m.spacing_m * (az - m.axis_deg.to_radians()).cos() / atm.c_mps
    });
    for (ch, buf) in out.iter_mut().enumerate() {
        let t_ch = if ch == 0 { t - half_lead } else { t + half_lead };
        ricker(buf, t_ch, amp, f0);
    }
}

fn ricker(out: &mut [f32], t: f64, amp: f64, f0: f64) {
    let center = t * ANALYSIS_FS;
    let half = 1.5 * ANALYSIS_FS / f0;
    let lo = (center - half).ceil().max(0.0) as usize;
    let hi = ((center + half).floor().max(0.0) as usize).min(out.len().saturating_sub(1));
    for i in lo..=hi {
        let tau = std::f64::consts::PI * f0 * (i as f64 - center) / ANALYSIS_FS;
        out[i] += (amp * (1.0 - 2.0 * tau * tau) * (-tau * tau).exp()) as f32;
    }
}

fn jitter(s: &Strike, unit: &Normal<f64>, rng: &mut impl Rng) -> (f64, f64) {
    let th = (s.ellipse_angle_deg as f64).to_radians();
    let a = s.semi_major_km as f64 / 1.1774 * unit.sample(rng);
    let b = s.semi_minor_km as f64 / 1.1774 * unit.sample(rng);
    (a * th.sin() + b * th.cos(), a * th.cos() - b * th.sin())
}

/// Slowly varying band-limited rain floor plus passing-car bursts (mostly common to both mics).
fn add_noise(out: &mut [Vec<f32>], sc: &Scenario, rng: &mut impl Rng) {
    let n = out[0].len();
    let common = band_noise(n, rng);
    let unit = Normal::new(0.0, 1.0).unwrap();
    let cars: Vec<f64> = (0..sc.car_passes).map(|_| rng.gen_range(0.0..sc.duration_s)).collect();
    let mut level = 1.0f64;
    let mut gain = Vec::with_capacity(n);
    for i in 0..n {
        if i % 200 == 0 {
            level = (level + 0.05 * unit.sample(rng)).clamp(0.5, 1.5);
        }
        let t = i as f64 / ANALYSIS_FS;
        let car: f64 = cars.iter().map(|c| 3.0 * (-0.5 * ((t - c) / 3.0).powi(2)).exp()).sum();
        gain.push((sc.noise_level * (level + car)) as f32);
    }
    let mix = if out.len() > 1 { NOISE_DECORRELATION } else { 0.0 };
    for buf in out.iter_mut() {
        let own = if mix > 0.0 { band_noise(n, rng) } else { vec![0.0; n] };
        for i in 0..n {
            buf[i] += gain[i] * ((1.0 - mix) * common[i] + mix * own[i]);
        }
    }
}

/// Unit-RMS Gaussian noise band-limited to 20–400 Hz.
fn band_noise(n: usize, rng: &mut impl Rng) -> Vec<f32> {
    let unit = Normal::new(0.0, 1.0).unwrap();
    let mut noise: Vec<f32> = (0..n).map(|_| unit.sample(rng) as f32).collect();
    bandpass_zero_phase(&mut noise, 20.0, 400.0, ANALYSIS_FS);
    let rms = (noise.iter().map(|v| v * v).sum::<f32>() / n as f32).sqrt().max(1e-9);
    noise.iter_mut().for_each(|v| *v /= rms);
    noise
}

/// Writes 1 or 2 equal-length channels as float WAV at `ANALYSIS_FS`.
pub fn write_wav(path: &Path, channels: &[Vec<f32>]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: channels.len() as u16,
        sample_rate: ANALYSIS_FS as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for i in 0..channels[0].len() {
        for ch in channels {
            w.write_sample(ch[i])?;
        }
    }
    w.finalize()?;
    Ok(())
}
