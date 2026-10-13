//! Per-strike loudness refit: at a fixed lag, each strike's area becomes a free log-gain θᵢ around its
//! modeled value, so position is judged by timing rather than by the (uncertain) loudness model.
//! Maximises −(n_eff/2)·ln RSS − Σ(dᵢ − d̄)²/(2σ²), dᵢ = θᵢ − ln areaᵢ (prior on relative loudness only),
//! with the affine fit (α, β) to the template profiled out. Levenberg–Marquardt on θ.

use crate::model::render::Kernel;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FreeAmp {
    /// Top lags per cell that get refit.
    pub lags: usize,
    /// Prior std-dev of a strike's log-loudness around the model (nats).
    pub sigma: f64,
}

/// Result at one lag: the refit correlation and the prior penalty paid for it (nats).
pub struct Refit {
    pub rho: f64,
    pub penalty: f64,
}

struct Active<'a> {
    off: usize,
    k: &'a [f64],
    theta0: f64,
}

const MAX_ITERS: usize = 25;

/// `template` is the centred, unit-norm observation; the window is model samples `lag..lag + template.len()`.
pub fn refit(template: &[f32], lag: usize, kernels: &[Kernel], exponent: f64, n_eff: f64, cfg: FreeAmp) -> Option<Refit> {
    let l = template.len();
    let active: Vec<Active> = kernels
        .iter()
        .filter_map(|kn| {
            let a = kn.start.max(lag);
            let b = (kn.start + kn.values.len()).min(lag + l);
            (a < b).then(|| Active { off: a - lag, k: &kn.values[a - kn.start..b - kn.start], theta0: kn.area.ln() })
        })
        .collect();
    let n = active.len();
    if n == 0 {
        return None;
    }
    let v: Vec<f64> = template.iter().map(|&x| x as f64).collect();
    let mut theta: Vec<f64> = active.iter().map(|a| a.theta0).collect();
    let mut st = State::default();
    let mut f = st.eval(&v, &active, &theta, exponent, n_eff, cfg.sigma)?;
    let mut mu = 1e-3;
    for _ in 0..MAX_ITERS {
        let (h, grad) = st.normal_equations(&active, &theta, exponent, n_eff, cfg.sigma);
        let mut improved = false;
        while mu < 1e8 {
            let mut a = h.clone();
            (0..n).for_each(|i| a[i * n + i] += mu * (h[i * n + i] + 1e-9));
            let mut step: Vec<f64> = grad.iter().map(|g| -g).collect();
            if !cholesky_solve(&mut a, n, &mut step) {
                mu *= 4.0;
                continue;
            }
            let trial: Vec<f64> = theta.iter().zip(&step).map(|(t, s)| t + s).collect();
            let mut st2 = State::default();
            match st2.eval(&v, &active, &trial, exponent, n_eff, cfg.sigma) {
                Some(f2) if f2 < f => {
                    let gain = f - f2;
                    (theta, st, f) = (trial, st2, f2);
                    mu = (mu / 3.0).max(1e-9);
                    improved = gain > 1e-4;
                    break;
                }
                _ => mu *= 4.0,
            }
        }
        if !improved {
            break;
        }
    }
    let rho = (1.0 - st.rss).max(0.0).sqrt();
    Some(Refit { rho, penalty: st.penalty })
}

#[derive(Default)]
struct State {
    /// Uncompressed model power over the window.
    power: Vec<f64>,
    resid: Vec<f64>,
    beta: f64,
    rss: f64,
    penalty: f64,
    dev: Vec<f64>,
}

impl State {
    /// Objective to minimise; None when the best affine fit is anti-correlated.
    fn eval(&mut self, v: &[f64], act: &[Active], theta: &[f64], g: f64, n_eff: f64, sigma: f64) -> Option<f64> {
        let l = v.len();
        self.power = vec![0.0; l];
        for (a, t) in act.iter().zip(theta) {
            let w = t.exp();
            self.power[a.off..a.off + a.k.len()].iter_mut().zip(a.k).for_each(|(p, k)| *p += w * k);
        }
        let m: Vec<f64> = self.power.iter().map(|p| p.max(0.0).powf(g)).collect();
        let mean = m.iter().sum::<f64>() / l as f64;
        let (sxy, sxx) = m.iter().zip(v).fold((0.0, 0.0), |(xy, xx), (mi, vi)| (xy + (mi - mean) * vi, xx + (mi - mean).powi(2)));
        if sxx <= 0.0 || sxy <= 0.0 {
            return None;
        }
        self.beta = sxy / sxx;
        self.resid = m.iter().zip(v).map(|(mi, vi)| vi - self.beta * (mi - mean)).collect();
        self.rss = self.resid.iter().map(|r| r * r).sum::<f64>().max(1e-12);
        self.dev = act.iter().zip(theta).map(|(a, t)| t - a.theta0).collect();
        let dbar = self.dev.iter().sum::<f64>() / act.len() as f64;
        self.penalty = self.dev.iter().map(|d| (d - dbar).powi(2)).sum::<f64>() / (2.0 * sigma * sigma);
        Some(0.5 * n_eff * self.rss.ln() + self.penalty)
    }

    /// Gauss–Newton Hessian and gradient of the objective at the evaluated θ (α, β held fixed).
    fn normal_equations(&self, act: &[Active], theta: &[f64], g: f64, n_eff: f64, sigma: f64) -> (Vec<f64>, Vec<f64>) {
        let n = act.len();
        // dr_t/dθ_i = −w_t·e^{θ_i}·K_i(t), with w_t = β·g·M_t^{g−1}.
        let w: Vec<f64> = self.power.iter().map(|&p| if p > 0.0 { self.beta * g * p.powf(g - 1.0) } else { 0.0 }).collect();
        let scale = n_eff / self.rss;
        let mut h = vec![0.0; n * n];
        let mut grad = vec![0.0; n];
        for i in 0..n {
            let (ai, ei) = (&act[i], theta[i].exp());
            let jr: f64 = ai.k.iter().enumerate().map(|(t, k)| w[ai.off + t] * k * self.resid[ai.off + t]).sum();
            grad[i] = -scale * ei * jr;
            for j in i..n {
                let aj = &act[j];
                let (lo, hi) = (ai.off.max(aj.off), (ai.off + ai.k.len()).min(aj.off + aj.k.len()));
                if lo >= hi {
                    continue;
                }
                let s: f64 = (lo..hi).map(|t| w[t] * w[t] * ai.k[t - ai.off] * aj.k[t - aj.off]).sum();
                let v = scale * ei * theta[j].exp() * s;
                h[i * n + j] = v;
                h[j * n + i] = v;
            }
        }
        let dbar = self.dev.iter().sum::<f64>() / n as f64;
        let inv = 1.0 / (sigma * sigma);
        for i in 0..n {
            grad[i] += (self.dev[i] - dbar) * inv;
            for j in 0..n {
                h[i * n + j] += inv * (if i == j { 1.0 } else { 0.0 } - 1.0 / n as f64);
            }
        }
        (h, grad)
    }
}

/// Solves A·x = b in place for symmetric positive-definite A (row-major n×n); false if not SPD.
fn cholesky_solve(a: &mut [f64], n: usize, b: &mut [f64]) -> bool {
    for j in 0..n {
        let d = a[j * n + j] - (0..j).map(|k| a[j * n + k].powi(2)).sum::<f64>();
        if d <= 0.0 || !d.is_finite() {
            return false;
        }
        let d = d.sqrt();
        a[j * n + j] = d;
        for i in j + 1..n {
            let s = a[i * n + j] - (0..j).map(|k| a[i * n + k] * a[j * n + k]).sum::<f64>();
            a[i * n + j] = s / d;
        }
    }
    for i in 0..n {
        b[i] = (b[i] - (0..i).map(|k| a[i * n + k] * b[k]).sum::<f64>()) / a[i * n + i];
    }
    for i in (0..n).rev() {
        b[i] = (b[i] - (i + 1..n).map(|k| a[k * n + i] * b[k]).sum::<f64>()) / a[i * n + i];
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two pulses whose true loudness ratio is flipped vs the prior: the refit must recover most of it.
    #[test]
    fn recovers_swapped_loudness() {
        let pulse = |c: usize| -> Vec<f64> { (0..60).map(|t| (-((t as f64 - c as f64) / 4.0).powi(2)).exp() / 7.09).collect() };
        let kernels = vec![
            Kernel { start: 0, values: pulse(20), area: 4.0 },
            Kernel { start: 40, values: pulse(20), area: 1.0 },
        ];
        let truth: Vec<f64> = (0..100)
            .map(|t| {
                let a = if t < 60 { kernels[0].values[t] } else { 0.0 };
                let b = if (40..100).contains(&t) { kernels[1].values[t - 40] } else { 0.0 };
                (1.0 * a + 4.0 * b).sqrt()
            })
            .collect();
        let mean = truth.iter().sum::<f64>() / 100.0;
        let norm = truth.iter().map(|x| (x - mean).powi(2)).sum::<f64>().sqrt();
        let tpl: Vec<f32> = truth.iter().map(|x| ((x - mean) / norm) as f32).collect();
        let r = refit(&tpl, 0, &kernels, 0.5, 50.0, FreeAmp { lags: 1, sigma: 3.0 }).unwrap();
        assert!(r.rho > 0.99, "rho {}", r.rho);
    }
}
