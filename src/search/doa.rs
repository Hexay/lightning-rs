//! Stereo direction evidence. Each usable window's measured L−R lag is regressed on the
//! bearing (cos, sin) of the strike that dominates the window for this candidate:
//! lag ≈ c₀ + a·cos(az) + b·sin(az), where (a, b) = mic spacing/c along the unknown mic axis and c₀
//! absorbs any hardware offset. Evidence is the tempered log-likelihood ratio against "lag is
//! constant", BIC-penalised for (a, b). A one-sided storm gives no bearing variation, hence ≈ 0.

use crate::audio::stereo::LagSample;
use crate::model::propagation::Arrival;

/// Adjacent windows overlap by half, so each carries about half an independent sample.
const WINDOW_INDEPENDENCE: f64 = 0.5;
const MIN_WINDOWS: usize = 8;
/// A window counts only when one strike supplies at least this share of its modeled energy.
const MIN_DOMINANCE: f64 = 0.6;
/// Upper bound on |(a, b)|: ~0.35 m mic spacing.
const MAX_LEAD_S: f64 = 0.001;

/// Log-likelihood ratio (nats) of the measured lag track given arrivals for a clip starting at `start`.
pub fn log_lik(lags: &[LagSample], window_s: f64, arrivals: &[Arrival], start: f64) -> f64 {
    let mut dominant = Vec::new();
    let rows: Vec<(f64, f64, f64)> = lags
        .iter()
        .filter_map(|w| {
            let (t0, t1) = (start + w.t - 0.5 * window_s, start + w.t + 0.5 * window_s);
            let (mut best, mut pmax, mut sp) = (0, 0.0, 0.0);
            for (i, a) in arrivals.iter().enumerate() {
                let p = a.area * window_fraction(a, t0, t1);
                if p > pmax {
                    (best, pmax) = (i, p);
                }
                sp += p;
            }
            // GCC locks onto the loudest source; mixed windows have no single bearing.
            if sp <= 0.0 || pmax < MIN_DOMINANCE * sp {
                return None;
            }
            dominant.push(best);
            let az = arrivals[best].bearing_rad;
            Some((az.cos(), az.sin(), w.lag_s))
        })
        .collect();
    let n = rows.len();
    if n < MIN_WINDOWS {
        return 0.0;
    }
    dominant.sort_unstable();
    dominant.dedup();
    let mean = rows.iter().map(|r| r.2).sum::<f64>() / n as f64;
    let rss0: f64 = rows.iter().map(|r| (r.2 - mean).powi(2)).sum::<f64>().max(1e-30);
    let Some((c0, a, b)) = fit(&rows) else { return 0.0 };
    if a.hypot(b) > MAX_LEAD_S {
        return 0.0;
    }
    let rss1: f64 = rows.iter().map(|&(x, y, l)| (l - c0 - a * x - b * y).powi(2)).sum::<f64>().max(1e-30);
    // One strike's rumble spans many windows: independent bearings ≈ distinct dominant strikes.
    let n_eff = (WINDOW_INDEPENDENCE * n as f64).min(dominant.len() as f64);    (-0.5 * n_eff * (rss1 / rss0).ln() - n_eff.ln()).max(0.0)
}

/// Fraction of an arrival's (exponential-tail) energy falling in [t0, t1].
fn window_fraction(a: &Arrival, t0: f64, t1: f64) -> f64 {
    if t1 <= a.onset {
        return 0.0;
    }
    let tau = a.decay_s.max(1e-3);
    let cdf = |t: f64| if t <= a.onset { 0.0 } else { 1.0 - (-(t - a.onset) / tau).exp() };
    cdf(t1) - cdf(t0)
}

/// Least squares for l = c0 + a·x + b·y.
fn fit(rows: &[(f64, f64, f64)]) -> Option<(f64, f64, f64)> {
    let mut m = [[0.0f64; 3]; 3];
    let mut v = [0.0f64; 3];
    for &(x, y, l) in rows {
        let f = [1.0, x, y];
        for i in 0..3 {
            v[i] += f[i] * l;
            for j in 0..3 {
                m[i][j] += f[i] * f[j];
            }
        }
    }
    solve3(m, v)
}

fn solve3(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<(f64, f64, f64)> {
    let det = |m: &[[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(&m);
    if d.abs() < 1e-12 * m[0][0].powi(3).max(1e-300) {
        return None;
    }
    let col = |k: usize| {
        let mut c = m;
        (0..3).for_each(|i| c[i][k] = v[i]);
        det(&c) / d
    };
    Some((col(0), col(1), col(2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arr(onset: f64, bearing_deg: f64) -> Arrival {
        Arrival { onset, decay_s: 2.0, sigma_s: 0.1, area: 1.0, audible: true, bearing_rad: bearing_deg.to_radians() }
    }

    /// Strikes all around the listener, lag generated from a 0.15 m pair at axis 40°: strong evidence.
    #[test]
    fn surrounding_storm_is_informative() {
        let arrivals: Vec<Arrival> = (0..40).map(|i| arr(10.0 * i as f64, (i * 47 % 360) as f64)).collect();
        let lags: Vec<LagSample> = (0..80)
            .map(|k| {
                let t = 5.0 * k as f64 + 1.0;
                let a = &arrivals[(k / 2).min(39)];
                let lag = 0.15 / 340.0 * (a.bearing_rad - 40f64.to_radians()).cos() + 1e-5 * ((k * 7 % 5) as f64 - 2.0);
                LagSample { t, lag_s: lag, coherence: 0.95, energy: 1.0 }
            })
            .collect();
        assert!(log_lik(&lags, 1.0, &arrivals, 0.0) > 20.0);
        // Same lags, but candidate hears everything from one bearing: no evidence.
        let one_sided: Vec<Arrival> = arrivals.iter().map(|a| Arrival { bearing_rad: 0.3, ..*a }).collect();
        assert_eq!(log_lik(&lags, 1.0, &one_sided, 0.0), 0.0);
    }
}
