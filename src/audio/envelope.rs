use std::collections::VecDeque;

/// RMS of `x` (sampled at `fs`) in frames centred every 1/`rate` s, window two hops wide.
pub fn rms_envelope(x: &[f32], fs: f64, rate: f64) -> Vec<f32> {
    let hop = fs / rate;
    let half = hop.round().max(1.0) as usize;
    let n = (x.len() as f64 / hop).floor() as usize;
    let mut sq = vec![0.0f64; x.len() + 1];
    for (i, v) in x.iter().enumerate() {
        sq[i + 1] = sq[i] + (*v as f64) * (*v as f64);
    }
    (0..n)
        .map(|k| {
            let c = ((k as f64 + 0.5) * hop) as usize;
            let (lo, hi) = (c.saturating_sub(half), (c + half).min(x.len()));
            (((sq[hi] - sq[lo]) / (hi - lo).max(1) as f64).sqrt()) as f32
        })
        .collect()
}

pub fn gaussian_smooth(x: &[f32], sigma_samples: f64) -> Vec<f32> {
    if sigma_samples < 0.3 {
        return x.to_vec();
    }
    let r = (4.0 * sigma_samples).ceil() as isize;
    let w: Vec<f64> = (-r..=r).map(|i| (-0.5 * (i as f64 / sigma_samples).powi(2)).exp()).collect();
    let n = x.len() as isize;
    (0..n)
        .map(|i| {
            let (mut acc, mut norm) = (0.0, 0.0);
            for (k, wk) in w.iter().enumerate() {
                let j = i + k as isize - r;
                if (0..n).contains(&j) {
                    acc += wk * x[j as usize] as f64;
                    norm += wk;
                }
            }
            (acc / norm) as f32
        })
        .collect()
}

/// Subtracts a running minimum over `window` samples (removes slowly varying rain/wind floor).
pub fn subtract_rolling_min(x: &[f32], window: usize) -> Vec<f32> {
    let half = window / 2;
    let mut out = Vec::with_capacity(x.len());
    let mut dq: VecDeque<usize> = VecDeque::new();
    let mut next = 0;
    for i in 0..x.len() {
        let hi = (i + half).min(x.len() - 1);
        while next <= hi {
            while dq.back().is_some_and(|&b| x[b] >= x[next]) {
                dq.pop_back();
            }
            dq.push_back(next);
            next += 1;
        }
        while dq.front().is_some_and(|&f| f + half < i) {
            dq.pop_front();
        }
        out.push(x[i] - x[*dq.front().unwrap()]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_min_removes_floor() {
        let x: Vec<f32> = (0..100).map(|i| 1.0 + if i == 50 { 5.0 } else { 0.0 }).collect();
        let y = subtract_rolling_min(&x, 10);
        assert_eq!(y[10], 0.0);
        assert_eq!(y[50], 5.0);
    }

    #[test]
    fn rms_of_constant_tone() {
        let fs = 2000.0;
        let x: Vec<f32> = (0..20000).map(|i| (i as f32 * 0.3).sin()).collect();
        let e = rms_envelope(&x, fs, 10.0);
        assert_eq!(e.len(), 100);
        assert!((e[50] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.02);
    }
}
