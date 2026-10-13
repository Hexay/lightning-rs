use super::emg::{emg, gaussian};
use super::propagation::Arrival;

/// A modeled envelope sampled at `rate` Hz starting at epoch `origin`.
#[derive(Default)]
pub struct Rendered {
    pub samples: Vec<f32>,
    /// Sample index of each audible onset, ascending.
    pub onset_idx: Vec<usize>,
}

/// Rumble tail is cut after this many decay constants (e^-6 ≈ 0.25%).
const TAIL_TAUS: f64 = 6.0;

/// Renders arrivals into `out` (len samples), blurring each by `smooth_s` (matches the observed smoothing),
/// then raises the sum to `exponent`.
pub fn render(arrivals: &[Arrival], origin: f64, rate: f64, len: usize, smooth_s: f64, exponent: f64, out: &mut Rendered) {
    out.samples.clear();
    out.samples.resize(len, 0.0);
    out.onset_idx.clear();
    let end = origin + len as f64 / rate;
    for a in arrivals {
        if trace(a, origin, rate, len, smooth_s, |i, v| out.samples[i] += v as f32).is_none() {
            continue;
        }
        if a.audible && a.onset >= origin && a.onset < end {
            out.onset_idx.push(((a.onset - origin) * rate) as usize);
        }
    }
    out.onset_idx.sort_unstable();
    compress(&mut out.samples, exponent);
}

/// One arrival's uncompressed, unit-area pulse over sample range `start..start + values.len()`.
pub struct Kernel {
    pub start: usize,
    pub values: Vec<f64>,
    /// The arrival's modeled area (its prior amplitude).
    pub area: f64,
}

/// Per-arrival unit-area kernels on the same grid as `render`; arrivals outside the buffer are skipped.
pub fn render_kernels(arrivals: &[Arrival], origin: f64, rate: f64, len: usize, smooth_s: f64, out: &mut Vec<Kernel>) {
    out.clear();
    for a in arrivals.iter().filter(|a| a.area > 0.0) {
        let unit = Arrival { area: 1.0, ..*a };
        let mut values = Vec::new();
        if let Some(start) = trace(&unit, origin, rate, len, smooth_s, |_, v| values.push(v)) {
            out.push(Kernel { start, values, area: a.area });
        }
    }
}

/// Calls `f(sample_index, value)` over an arrival's support (ascending, contiguous); returns the first index.
fn trace(a: &Arrival, origin: f64, rate: f64, len: usize, smooth_s: f64, mut f: impl FnMut(usize, f64)) -> Option<usize> {
    let end = origin + len as f64 / rate;
    let sigma = a.sigma_s.hypot(smooth_s).max(0.5 / rate);
    // Past `exact_until` erfc(z) ≈ 2 (z < -3), so the EMG is a pure exponential and can be stepped.
    let (exact_until, hi_t) = if a.decay_s > 0.0 {
        let t = a.onset + sigma * sigma / a.decay_s + 4.3 * sigma;
        (t, t + TAIL_TAUS * a.decay_s)
    } else {
        (a.onset + 4.0 * sigma, a.onset + 4.0 * sigma)
    };
    let lo_t = a.onset - 4.0 * sigma;
    if hi_t < origin || lo_t > end {
        return None;
    }
    let lo = ((lo_t - origin) * rate).floor().max(0.0) as usize;
    let hi = (((hi_t - origin) * rate).ceil() as usize).min(len);
    let step = (-1.0 / (a.decay_s.max(1e-9) * rate)).exp();
    let mut prev = 0.0f64;
    for i in lo..hi {
        let t = origin + i as f64 / rate;
        let val = if a.decay_s <= 0.0 {
            a.area * gaussian(t, a.onset, sigma)
        } else if t < exact_until || i == lo {
            a.area * emg(t, a.onset, sigma, a.decay_s)
        } else {
            prev * step
        };
        prev = val;
        f(i, val);
    }
    Some(lo)
}

/// x^exponent for x ≥ 0, with sqrt fast paths for the exponents the model actually uses.
fn compress(x: &mut [f32], exponent: f64) {
    match exponent {
        1.0 => {}
        0.5 => x.iter_mut().for_each(|v| *v = v.max(0.0).sqrt()),
        0.25 => x.iter_mut().for_each(|v| *v = v.max(0.0).sqrt().sqrt()),
        e => {
            let g = e as f32;
            x.iter_mut().for_each(|v| *v = v.max(0.0).powf(g));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepped_tail_matches_exact_emg() {
        let a = Arrival { onset: 10.0, decay_s: 2.0, sigma_s: 0.3, area: 1.0, audible: true };
        let (rate, smooth) = (10.0, 0.5);
        let mut out = Rendered::default();
        render(&[a], 0.0, rate, 400, smooth, 1.0, &mut out);
        let sigma = 0.3f64.hypot(smooth);
        for i in [95usize, 105, 130, 160, 200] {
            let exact = emg(i as f64 / rate, 10.0, sigma, 2.0);
            assert!((out.samples[i] as f64 - exact).abs() < 1e-4 * exact.max(1e-3), "i={i}");
        }
    }
}
