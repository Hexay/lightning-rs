use crate::audio::envelope::{gaussian_smooth, rms_envelope, subtract_rolling_min};
use crate::audio::{ANALYSIS_FS, BandAudio};

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Onset {
    /// Seconds from recording start.
    pub t: f64,
    pub strength: f64,
}

const RATE: f64 = 50.0;
const SMOOTH_S: f64 = 0.1;
const FLOOR_WINDOW_S: f64 = 30.0;
const DIFF_S: f64 = 0.2;
const MIN_GAP_S: f64 = 2.0;
const MAD_K: f64 = 6.0;

/// Thunder onsets: peaks of the rectified envelope rise, above a robust (median + k·MAD) threshold.
pub fn detect(audio: &BandAudio, max_onsets: usize) -> Vec<Onset> {
    let env = rms_envelope(&audio.samples, ANALYSIS_FS, RATE);
    let env = gaussian_smooth(&env, SMOOTH_S * RATE);
    let env = subtract_rolling_min(&env, (FLOOR_WINDOW_S * RATE) as usize);
    let lag = (DIFF_S * RATE) as usize;
    let rise: Vec<f64> = (0..env.len()).map(|i| if i < lag { 0.0 } else { (env[i] - env[i - lag]).max(0.0) as f64 }).collect();

    let thresh = robust_threshold(&rise);
    let gap = (MIN_GAP_S * RATE) as usize;
    let mut peaks: Vec<Onset> = (0..rise.len())
        .filter(|&i| {
            rise[i] > thresh && {
                let (lo, hi) = (i.saturating_sub(gap), (i + gap + 1).min(rise.len()));
                rise[lo..hi].iter().all(|&v| v <= rise[i])
            }
        })
        // Steepest rise lags the true onset by about half the differencing span.
        .map(|i| Onset { t: (i as f64 - lag as f64 / 2.0) / RATE, strength: rise[i] })
        .collect();
    peaks.sort_by(|a, b| b.strength.total_cmp(&a.strength));
    peaks.truncate(max_onsets);
    peaks.sort_by(|a, b| a.t.total_cmp(&b.t));
    peaks
}

fn robust_threshold(x: &[f64]) -> f64 {
    let mut v = x.to_vec();
    v.sort_by(f64::total_cmp);
    let med = v[v.len() / 2];
    let mut dev: Vec<f64> = v.iter().map(|a| (a - med).abs()).collect();
    dev.sort_by(f64::total_cmp);
    med + MAD_K * 1.4826 * dev[dev.len() / 2].max(1e-12)
}
