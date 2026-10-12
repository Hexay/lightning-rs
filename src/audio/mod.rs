pub mod decode;
pub mod envelope;
pub mod filter;

use anyhow::Result;
use std::path::Path;

/// Analysis sample rate: comfortably above the 250 Hz band edge, cheap to filter.
pub const ANALYSIS_FS: f64 = 2000.0;

/// Thunder band-limited mono audio at `ANALYSIS_FS`.
pub struct BandAudio {
    pub samples: Vec<f32>,
    pub duration_s: f64,
}

pub fn load_band(path: &Path, lo_hz: f64, hi_hz: f64) -> Result<BandAudio> {
    let mut samples = decode::decode(path, ANALYSIS_FS as u32, 1)?;
    anyhow::ensure!(!samples.is_empty(), "no audio decoded from {}", path.display());
    filter::bandpass_zero_phase(&mut samples, lo_hz, hi_hz, ANALYSIS_FS);
    let duration_s = samples.len() as f64 / ANALYSIS_FS;
    Ok(BandAudio { samples, duration_s })
}

/// Observed envelope at `rate` Hz, smoothed by `smooth_s`, with the noise floor optionally removed.
pub fn observed_envelope(audio: &BandAudio, rate: f64, smooth_s: f64, floor_window_s: Option<f64>) -> Vec<f32> {
    let env = envelope::rms_envelope(&audio.samples, ANALYSIS_FS, rate);
    let env = envelope::gaussian_smooth(&env, smooth_s * rate);
    match floor_window_s {
        Some(w) => envelope::subtract_rolling_min(&env, (w * rate) as usize),
        None => env,
    }
}
