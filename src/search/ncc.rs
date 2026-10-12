use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

/// Observed envelope prepared for sliding correlation against long model buffers (overlap-save FFT).
pub struct Template {
    pub len: usize,
    /// Pearson (zero-mean template, variance-normalised windows) vs plain cosine.
    pub centered: bool,
    fft_size: usize,
    step: usize,
    spectrum: Vec<Complex32>,
    fft: Arc<dyn Fft<f32>>,
    ifft: Arc<dyn Fft<f32>>,
}

#[derive(Default)]
pub struct Scratch {
    buf: Vec<Complex32>,
    fft_scratch: Vec<Complex32>,
    pub num: Vec<f32>,
}

impl Template {
    pub fn new(observed: &[f32], centered: bool) -> Self {
        let mut o: Vec<f64> = observed.iter().map(|&v| v as f64).collect();
        if centered {
            let mean = o.iter().sum::<f64>() / o.len() as f64;
            o.iter_mut().for_each(|v| *v -= mean);
        }
        let norm = o.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-30);
        let len = o.len();
        let fft_size = (4 * len).next_power_of_two();
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let ifft = planner.plan_fft_inverse(fft_size);
        // Correlation = convolution with the time-reversed template.
        let mut spectrum = vec![Complex32::default(); fft_size];
        for (i, v) in o.iter().enumerate() {
            spectrum[len - 1 - i] = Complex32::new((v / norm) as f32, 0.0);
        }
        fft.process(&mut spectrum);
        let scale = 1.0 / fft_size as f32;
        spectrum.iter_mut().for_each(|c| *c *= scale);
        Self { len, centered, fft_size, step: fft_size - len + 1, spectrum, fft, ifft }
    }

    /// num[k] = Σ_t model[k+t]·ô[t] for every lag k with the window inside `model`.
    /// Blocks whose model segment is all zero are skipped (left at 0).
    pub fn correlate(&self, model: &[f32], prefix_abs: &[f64], s: &mut Scratch) {
        let n_lags = (model.len() + 1).saturating_sub(self.len);
        s.num.clear();
        s.num.resize(n_lags, 0.0);
        s.buf.resize(self.fft_size, Complex32::default());
        s.fft_scratch.resize(self.fft.get_inplace_scratch_len(), Complex32::default());
        let mut start = 0;
        while start < n_lags {
            let seg_end = (start + self.fft_size).min(model.len());
            if prefix_abs[seg_end] - prefix_abs[start] > 0.0 {
                for (i, c) in s.buf.iter_mut().enumerate() {
                    *c = Complex32::new(model.get(start + i).copied().filter(|_| start + i < seg_end).unwrap_or(0.0), 0.0);
                }
                self.fft.process_with_scratch(&mut s.buf, &mut s.fft_scratch);
                s.buf.iter_mut().zip(&self.spectrum).for_each(|(a, b)| *a *= b);
                self.ifft.process_with_scratch(&mut s.buf, &mut s.fft_scratch);
                let take = self.step.min(n_lags - start);
                for j in 0..take {
                    s.num[start + j] = s.buf[self.len - 1 + j].re;
                }
            }
            start += self.step;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_direct_correlation() {
        let obs: Vec<f32> = (0..50).map(|i| ((i * 7) % 11) as f32).collect();
        let mut model = vec![0.0f32; 3000];
        for (i, v) in model.iter_mut().enumerate().skip(400).take(1500) {
            *v = ((i * 13) % 17) as f32;
        }
        let t = Template::new(&obs, true);
        let mut prefix = vec![0.0f64; model.len() + 1];
        for (i, v) in model.iter().enumerate() {
            prefix[i + 1] = prefix[i] + v.abs() as f64;
        }
        let mut s = Scratch::default();
        t.correlate(&model, &prefix, &mut s);
        let mean = obs.iter().sum::<f32>() / 50.0;
        let norm = obs.iter().map(|v| (v - mean).powi(2)).sum::<f32>().sqrt();
        for k in [0usize, 380, 777, 1500, 2950] {
            let direct: f32 = (0..50).map(|j| model[k + j] * (obs[j] - mean) / norm).sum();
            assert!((s.num[k] - direct).abs() < 1e-2 * direct.abs().max(1.0), "lag {k}: {} vs {direct}", s.num[k]);
        }
    }
}
