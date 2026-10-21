use super::decode::decode;
use anyhow::Result;
use base64::Engine;
use rustfft::{FftPlanner, num_complex::Complex32};
use serde::Serialize;
use std::path::Path;

const FS: u32 = 2000;
const WIN: usize = 1024;
const HOP_S: f64 = 0.2;
const F_LO_HZ: f64 = 10.0;
const N_BANDS: usize = 96;
/// Colour range below the 99.5th-percentile level.
const RANGE_DB: f32 = 45.0;

/// Log-frequency power spectrogram, 8-bit dB-scaled, row-major (band, frame), for the viewer.
#[derive(Serialize)]
pub struct Spectrogram {
    /// Centre of frame 0, seconds after the clip start.
    pub t0_s: f64,
    pub hop_s: f64,
    pub frames: usize,
    pub freqs_hz: Vec<f64>,
    pub data_b64: String,
}

/// Decodes `path` again at full analysis bandwidth (the search audio is band-limited) and computes the spectrogram.
pub fn load(path: &Path) -> Result<Spectrogram> {
    let x = decode(path, FS, 1)?;
    let hop = (HOP_S * FS as f64) as usize;
    let frames = x.len().saturating_sub(WIN) / hop + 1;
    let fft = FftPlanner::<f32>::new().plan_fft_forward(WIN);
    let hann: Vec<f32> = (0..WIN).map(|i| (std::f32::consts::PI * i as f32 / WIN as f32).sin().powi(2)).collect();
    let nyq = FS as f64 / 2.0;
    let edges: Vec<f64> = (0..=N_BANDS).map(|k| F_LO_HZ * (nyq / F_LO_HZ).powf(k as f64 / N_BANDS as f64)).collect();
    let bin = |f: f64| ((f / nyq * (WIN / 2) as f64) as usize).min(WIN / 2 - 1);

    let mut db = vec![0f32; N_BANDS * frames];
    let mut buf = vec![Complex32::default(); WIN];
    for t in 0..frames {
        for (i, b) in buf.iter_mut().enumerate() {
            *b = Complex32::new(x.get(t * hop + i).copied().unwrap_or(0.0) * hann[i], 0.0);
        }
        fft.process(&mut buf);
        for k in 0..N_BANDS {
            let (lo, hi) = (bin(edges[k]), bin(edges[k + 1]).max(bin(edges[k]) + 1));
            let p = buf[lo..hi].iter().map(|c| c.norm_sqr()).sum::<f32>() / (hi - lo) as f32;
            db[k * frames + t] = 10.0 * (p + 1e-20).log10();
        }
    }
    let mut sorted = db.clone();
    sorted.sort_by(f32::total_cmp);
    let top = sorted[(sorted.len() as f64 * 0.995) as usize];
    let bytes: Vec<u8> = db.iter().map(|v| ((v - top + RANGE_DB) / RANGE_DB * 255.0).clamp(0.0, 255.0) as u8).collect();
    Ok(Spectrogram {
        t0_s: WIN as f64 / 2.0 / FS as f64,
        hop_s: HOP_S,
        frames,
        freqs_hz: edges.windows(2).map(|w| (w[0] * w[1]).sqrt()).collect(),
        data_b64: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}
