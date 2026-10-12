/// ln(erfc(z)) for z >= 0 (Numerical Recipes erfcc, rel. error < 1.2e-7), stable for large z.
fn ln_erfc_pos(z: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = -1.265_512_23
        + t * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98 + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    t.ln() - z * z + poly
}

/// Exponentially modified Gaussian density: Gaussian(mu, sigma) convolved with Exp(rate = 1/tau).
pub fn emg(t: f64, mu: f64, sigma: f64, tau: f64) -> f64 {
    let lam = 1.0 / tau;
    let z = (mu + lam * sigma * sigma - t) / (std::f64::consts::SQRT_2 * sigma);
    let expo = 0.5 * lam * (2.0 * mu + lam * sigma * sigma - 2.0 * t);
    if z >= 0.0 {
        0.5 * lam * (expo + ln_erfc_pos(z)).exp()
    } else {
        0.5 * lam * expo.exp() * (2.0 - ln_erfc_pos(-z).exp())
    }
}

pub fn gaussian(t: f64, mu: f64, sigma: f64) -> f64 {
    const INV_SQRT_2PI: f64 = 0.398_942_280_401_432_7;
    INV_SQRT_2PI / sigma * (-0.5 * ((t - mu) / sigma).powi(2)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emg_integrates_to_one_and_is_finite() {
        let (mu, sigma, tau) = (10.0, 0.3, 2.0);
        let dt = 0.001;
        let area: f64 = (0..40_000).map(|i| emg(i as f64 * dt, mu, sigma, tau) * dt).sum();
        assert!((area - 1.0).abs() < 1e-3, "area {area}");
        assert!(emg(0.0, 100.0, 0.05, 0.5).is_finite());
        assert!(emg(1000.0, 0.0, 0.05, 0.5) >= 0.0);
    }
}
