//! Inter-channel lag track: per short window, the L−R delay of the thunder band (GCC peak).
//! A two-mic recorder turns each strike's bearing into a delay ∝ cos(bearing − mic axis).

use super::{decode, filter};
use anyhow::Result;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex64;
use std::path::Path;

/// Decode rate for lag measurement: sub-millisecond delays need more than ANALYSIS_FS.
const LAG_FS: f64 = 8000.0;
const WINDOW_S: f64 = 1.0;
/// Phone/camera mic pairs are ≲ 0.3 m apart.
const MAX_LAG_S: f64 = 0.001;
/// Below this L/R correlation the window's lag is noise.
const MIN_COHERENCE: f64 = 0.8;

#[derive(Clone, Copy, Debug)]
pub struct LagSample {
    /// Window centre, seconds from clip start.
    pub t: f64,
    pub lag_s: f64,
    pub coherence: f64,
    pub energy: f64,
}

/// Usable windows only. None for mono (or channels identical to within rounding).
pub struct LagTrack {
    pub samples: Vec<LagSample>,
    pub window_s: f64,
}

pub fn load_lag_track(path: &Path, lo_hz: f64, hi_hz: f64) -> Result<Option<LagTrack>> {
    let inter = decode::decode(path, LAG_FS as u32, 2)?;
    let (mut l, mut r): (Vec<f32>, Vec<f32>) = inter.chunks_exact(2).map(|c| (c[0], c[1])).unzip();
    let diff: f64 = l.iter().zip(&r).map(|(a, b)| ((a - b) as f64).powi(2)).sum();
    let tot: f64 = l.iter().map(|a| (*a as f64).powi(2)).sum();
    if diff <= 1e-6 * tot.max(1e-30) {
        return Ok(None);
    }
    filter::bandpass_zero_phase(&mut l, lo_hz, hi_hz, LAG_FS);
    filter::bandpass_zero_phase(&mut r, lo_hz, hi_hz, LAG_FS);
    Ok(Some(LagTrack { samples: usable(lag_samples(&l, &r)), window_s: WINDOW_S }))
}

fn lag_samples(l: &[f32], r: &[f32]) -> Vec<LagSample> {
    let w = (WINDOW_S * LAG_FS) as usize;
    let n = (2 * w).next_power_of_two();
    let max_k = (MAX_LAG_S * LAG_FS).ceil() as usize;
    let mut planner = FftPlanner::<f64>::new();
    let (fft, ifft) = (planner.plan_fft_forward(n), planner.plan_fft_inverse(n));
    let hann: Vec<f64> = (0..w).map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / w as f64).cos()).collect();
    let mut out = Vec::new();
    let mut start = 0;
    while start + w <= l.len() {
        let mut a: Vec<Complex64> = (0..n).map(|i| Complex64::new(if i < w { l[start + i] as f64 * hann[i] } else { 0.0 }, 0.0)).collect();
        let mut b: Vec<Complex64> = (0..n).map(|i| Complex64::new(if i < w { r[start + i] as f64 * hann[i] } else { 0.0 }, 0.0)).collect();
        let (ea, eb) = (a.iter().map(|c| c.re * c.re).sum::<f64>(), b.iter().map(|c| c.re * c.re).sum::<f64>());
        fft.process(&mut a);
        fft.process(&mut b);
        a.iter_mut().zip(&b).for_each(|(x, y)| *x *= y.conj());
        ifft.process(&mut a);
        let norm = (ea * eb).sqrt().max(1e-30) * n as f64;
        // c[k] for k in -max_k..=max_k; positive k = left lags right.
        let cc: Vec<f64> = (0..=2 * max_k).map(|j| a[(j + n - max_k) % n].re / norm).collect();
        let k = (0..cc.len()).max_by(|&x, &y| cc[x].total_cmp(&cc[y])).unwrap();
        let frac = if k > 0 && k + 1 < cc.len() {
            let (y0, y1, y2) = (cc[k - 1], cc[k], cc[k + 1]);
            0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2)
        } else {
            0.0
        };
        out.push(LagSample {
            t: (start + w / 2) as f64 / LAG_FS,
            lag_s: (k as f64 + frac - max_k as f64) / LAG_FS,
            coherence: cc[k],
            energy: ea + eb,
        });
        start += w / 2;
    }
    out
}

/// Windows worth fitting: coherent and above the median energy (thunder, not rain).
fn usable(all: Vec<LagSample>) -> Vec<LagSample> {
    let mut e: Vec<f64> = all.iter().map(|s| s.energy).collect();
    e.sort_by(f64::total_cmp);
    let median = e.get(e.len() / 2).copied().unwrap_or(0.0);
    all.into_iter().filter(|s| s.coherence >= MIN_COHERENCE && s.energy > median).collect()
}
