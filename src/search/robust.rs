/// Huber threshold in robust-scale units for positive residuals.
const HUBER_C: f64 = 2.0;
const IRLS_ITERS: usize = 6;

/// Correlation-like fit of `y ≈ a·x + b` (a ≥ 0) where observed excess over the model
/// (unmodelled sounds: cars, rain bursts) is Huber-downweighted but missing thunder is not.
/// Equals the Pearson correlation when no residual exceeds the threshold.
pub fn robust_rho(x: &[f32], y: &[f32], scratch: &mut Vec<f64>) -> f64 {
    let n = x.len();
    let mut w = vec![1.0f64; n];
    let mut fit = (0.0, 0.0);
    for _ in 0..IRLS_ITERS {
        fit = weighted_fit(x, y, &w);
        scratch.clear();
        scratch.extend((0..n).map(|i| (y[i] as f64 - fit.0 * x[i] as f64 - fit.1).abs()));
        let mid = n / 2;
        let (_, med, _) = scratch.select_nth_unstable_by(mid, f64::total_cmp);
        let thresh = HUBER_C * 1.4826 * med.max(1e-12);
        for i in 0..n {
            let r = y[i] as f64 - fit.0 * x[i] as f64 - fit.1;
            w[i] = if r > thresh { thresh / r } else { 1.0 };
        }
    }
    let (a, b) = fit;
    let sw: f64 = w.iter().sum();
    let ybar = (0..n).map(|i| w[i] * y[i] as f64).sum::<f64>() / sw;
    let (mut ss_res, mut ss_tot) = (0.0, 0.0);
    for i in 0..n {
        let (xi, yi) = (x[i] as f64, y[i] as f64);
        ss_res += w[i] * (yi - a * xi - b).powi(2);
        ss_tot += w[i] * (yi - ybar).powi(2);
    }
    if a <= 0.0 || ss_tot <= 0.0 { 0.0 } else { (1.0 - ss_res / ss_tot).max(0.0).sqrt() }
}

fn weighted_fit(x: &[f32], y: &[f32], w: &[f64]) -> (f64, f64) {
    let (mut sw, mut sx, mut sy, mut sxx, mut sxy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..x.len() {
        let (xi, yi, wi) = (x[i] as f64, y[i] as f64, w[i]);
        sw += wi;
        sx += wi * xi;
        sy += wi * yi;
        sxx += wi * xi * xi;
        sxy += wi * xi * yi;
    }
    let var = sxx - sx * sx / sw;
    let a = if var > 1e-30 { ((sxy - sx * sy / sw) / var).max(0.0) } else { 0.0 };
    (a, (sy - a * sx) / sw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pearson(x: &[f32], y: &[f32]) -> f64 {
        let n = x.len() as f64;
        let (mx, my) = (x.iter().sum::<f32>() as f64 / n, y.iter().sum::<f32>() as f64 / n);
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for i in 0..x.len() {
            let (dx, dy) = (x[i] as f64 - mx, y[i] as f64 - my);
            sxy += dx * dy;
            sxx += dx * dx;
            syy += dy * dy;
        }
        sxy / (sxx * syy).sqrt()
    }

    #[test]
    fn unmodelled_burst_is_discounted() {
        let x: Vec<f32> = (0..400).map(|i| ((i as f32) * 0.07).sin().max(0.0)).collect();
        let mut y: Vec<f32> = x.iter().enumerate().map(|(i, v)| v + 0.05 * ((i * 7 % 13) as f32 / 13.0)).collect();
        let clean = robust_rho(&x, &y, &mut Vec::new());
        assert!((clean - pearson(&x, &y)).abs() < 0.05, "clean {clean}");
        for v in &mut y[200..230] {
            *v += 3.0;
        }
        let robust = robust_rho(&x, &y, &mut Vec::new());
        assert!(robust > pearson(&x, &y) + 0.2, "robust {robust} vs pearson {}", pearson(&x, &y));
    }
}
