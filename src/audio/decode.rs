use anyhow::{Context, Result, ensure};
use std::path::Path;
use std::process::Command;

/// Decodes any ffmpeg-readable file to interleaved f32 PCM at `rate` Hz with `channels` channels.
pub fn decode(path: &Path, rate: u32, channels: u32) -> Result<Vec<f32>> {
    let out = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-ac", &channels.to_string(), "-ar", &rate.to_string(), "-f", "f32le", "-"])
        .output()
        .context("running ffmpeg (is it on PATH?)")?;
    ensure!(out.status.success(), "ffmpeg failed: {}", String::from_utf8_lossy(&out.stderr));
    Ok(out.stdout.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
}
