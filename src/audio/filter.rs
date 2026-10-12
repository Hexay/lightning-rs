use std::f64::consts::PI;

#[derive(Clone, Copy)]
struct Biquad {
    b: [f64; 3],
    a: [f64; 2],
}

impl Biquad {
    /// RBJ cookbook low/high-pass section.
    fn new(highpass: bool, cutoff_hz: f64, q: f64, fs: f64) -> Self {
        let w = 2.0 * PI * cutoff_hz / fs;
        let alpha = w.sin() / (2.0 * q);
        let cw = w.cos();
        let a0 = 1.0 + alpha;
        let b = if highpass {
            [(1.0 + cw) / 2.0, -(1.0 + cw), (1.0 + cw) / 2.0]
        } else {
            [(1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0]
        };
        Self { b: b.map(|x| x / a0), a: [-2.0 * cw / a0, (1.0 - alpha) / a0] }
    }

    fn run(&self, x: &mut [f32]) {
        let (mut z1, mut z2) = (0.0f64, 0.0f64);
        for v in x.iter_mut() {
            let input = *v as f64;
            let y = self.b[0] * input + z1;
            z1 = self.b[1] * input - self.a[0] * y + z2;
            z2 = self.b[2] * input - self.a[1] * y;
            *v = y as f32;
        }
    }
}

/// Q values of the two sections of a 4th-order Butterworth.
const BUTTER4_Q: [f64; 2] = [0.541_196_1, 1.306_563];

/// Zero-phase 4th-order Butterworth band-pass (forward-backward, so effectively 8th order).
pub fn bandpass_zero_phase(x: &mut [f32], lo_hz: f64, hi_hz: f64, fs: f64) {
    let sections: Vec<Biquad> = BUTTER4_Q
        .iter()
        .flat_map(|&q| [Biquad::new(true, lo_hz, q, fs), Biquad::new(false, hi_hz, q, fs)])
        .collect();
    for s in &sections {
        s.run(x);
    }
    x.reverse();
    for s in &sections {
        s.run(x);
    }
    x.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone_gain(freq: f64) -> f64 {
        let fs = 2000.0;
        let mut x: Vec<f32> = (0..8000).map(|i| (2.0 * PI * freq * i as f64 / fs).sin() as f32).collect();
        bandpass_zero_phase(&mut x, 20.0, 250.0, fs);
        let mid = &x[2000..6000];
        (mid.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / mid.len() as f64 * 2.0).sqrt()
    }

    #[test]
    fn passes_band_rejects_outside() {
        assert!((tone_gain(80.0) - 1.0).abs() < 0.05);
        assert!(tone_gain(600.0) < 0.01);
        assert!(tone_gain(5.0) < 0.01);
    }
}
