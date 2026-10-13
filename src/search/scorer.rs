use super::feature::Feature;
use super::free_amp::{FreeAmp, refit};
use std::time::Instant;
use super::ncc::{Scratch, Template};
use super::robust::robust_rho;
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::model::propagation::{Arrival, arrival};
use crate::model::render::{Kernel, Rendered, render, render_kernels};
use crate::strikes::StrikeIndex;

/// Resolution settings for one coarse-to-fine stage.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    pub spacing_km: f64,
    pub rate_hz: f64,
    pub smooth_s: f64,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct PointScore {
    /// log p(audio | location) − log p(audio | no storm), marginalised over recording start time.
    pub log_ev: f64,
    pub best_rho: f32,
    /// Epoch seconds the recording most likely started.
    pub best_start: f64,
    pub strikes_in_window: u32,
}

pub struct Scorer<'a> {
    pub index: &'a StrikeIndex,
    pub kind: ModelKind,
    pub phys: &'a Physics,
    pub stage: Stage,
    pub feature: Feature,
    pub template: Template,
    /// Recording start search range, epoch seconds.
    pub start_range: (f64, f64),
    pub min_strikes: usize,
    /// Top lags re-scored with the outlier-robust fit (0 = plain correlation only).
    pub robust_lags: usize,
    /// Per-strike loudness refit at the top lags (envelope feature only).
    pub free_amp: Option<FreeAmp>,
    /// Effective independent samples in the clip; scales correlation into log-likelihood.
    pub n_eff: f64,
}

/// Upper bound on travel time from the query radius plus a rumble tail.
const MAX_LEAD_S: f64 = 240.0;

#[derive(Default)]
pub struct Workspace {
    ids: Vec<u32>,
    arrivals: Vec<Arrival>,
    rendered: Rendered,
    p1: Vec<f64>,
    p2: Vec<f64>,
    ncc: Scratch,
    rho: Vec<f32>,
    count: Vec<u32>,
    lls: Vec<f64>,
    robust_scratch: Vec<f64>,
    kernels: Vec<Kernel>,
    /// Prior penalty (nats) paid by a lag's loudness refit; 0 where none ran.
    penalty: Vec<f64>,
    /// Cumulative seconds in [query+arrivals, render, prefix+correlate, reduce].
    pub phase_s: [f64; 4],
}

impl Scorer<'_> {
    pub fn radius_km(&self) -> f64 {
        match self.kind {
            ModelKind::Improved => self.phys.query_radius_km(),
            ModelKind::Baseline => crate::model::params::BASELINE_RADIUS_KM,
        }
    }

    pub fn score(&self, p: &LatLon, atm: &Atmosphere, ws: &mut Workspace) -> PointScore {
        let mut clock = Instant::now();
        let mut lap = |ws: &mut Workspace, phase: usize| {
            let now = Instant::now();
            ws.phase_s[phase] += (now - clock).as_secs_f64();
            clock = now;
        };
        self.index.within(p, self.radius_km(), &mut ws.ids);
        if ws.ids.len() < self.min_strikes {
            return PointScore::default();
        }
        let rate = self.stage.rate_hz;
        // Earliest strike whose sound (travel + rumble tail) can still reach the window.
        let t_lo = self.start_range.0 - MAX_LEAD_S;
        let t_hi = self.start_range.1 + self.template.len as f64 / rate;
        ws.arrivals.clear();
        ws.arrivals.extend(
            ws.ids
                .iter()
                .map(|&i| &self.index.strikes[i as usize])
                .filter(|s| s.time >= t_lo && s.time <= t_hi)
                .map(|s| arrival(self.kind, self.phys, atm, s, p)),
        );
        lap(ws, 0);

        let origin = (self.start_range.0 * rate).floor() / rate;
        let len = ((self.start_range.1 - origin) * rate).ceil() as usize + self.template.len;
        let exponent = if self.kind == ModelKind::Improved { self.phys.render_exponent() } else { 1.0 };
        render(&ws.arrivals, origin, rate, len, self.stage.smooth_s, exponent, &mut ws.rendered);
        if self.free_amp.is_some() {
            render_kernels(&ws.arrivals, origin, rate, len, self.stage.smooth_s, &mut ws.kernels);
        }
        self.feature.apply(&mut ws.rendered.samples, rate, self.stage.smooth_s);
        lap(ws, 1);
        if ws.rendered.onset_idx.len() < self.min_strikes {
            return PointScore::default();
        }

        let m = &ws.rendered.samples;
        prefix_sums(m, &mut ws.p1, &mut ws.p2);
        self.template.correlate(m, &ws.p1, &mut ws.ncc);
        lap(ws, 2);
        let s = self.reduce(origin, ws);
        lap(ws, 3);
        s
    }

    /// Diagnostic: the rendered model over the clip window starting at `start_range.0`.
    pub fn model_window(&self, p: &LatLon, atm: &Atmosphere, ws: &mut Workspace) -> Vec<f32> {
        self.score(p, atm, ws);
        ws.rendered.samples.iter().take(self.template.len).copied().collect()
    }

    /// Turns per-lag correlations into the best lag and the time-marginalised evidence.
    fn reduce(&self, origin: f64, ws: &mut Workspace) -> PointScore {
        self.lag_correlations(ws);
        if self.robust_lags > 0 {
            self.rescore_peaks(ws);
        }
        ws.penalty.clear();
        ws.penalty.resize(ws.rho.len(), 0.0);
        if let Some(fa) = self.free_amp {
            self.refit_peaks(ws, fa);
        }
        let mut best = PointScore::default();
        ws.lls.clear();
        for (k, &rho) in ws.rho.iter().enumerate() {
            if rho.is_nan() {
                continue;
            }
            let rho = (rho as f64).clamp(0.0, 0.999);
            ws.lls.push(-0.5 * self.n_eff * (1.0 - rho * rho).ln() - ws.penalty[k]);
            if rho as f32 > best.best_rho {
                best.best_rho = rho as f32;
                best.best_start = origin + k as f64 / self.stage.rate_hz;
                best.strikes_in_window = ws.count[k];
            }
        }
        best.log_ev = log_mean_exp_with_nulls(&ws.lls, ws.rho.len());
        best
    }

    /// FFT-correlation → per-lag correlation (NaN where the window has too few strikes or no energy).
    fn lag_correlations(&self, ws: &mut Workspace) {
        let (num, p1, p2, onsets) = (&ws.ncc.num, &ws.p1, &ws.p2, &ws.rendered.onset_idx);
        let l = self.template.len;
        let (mut lo, mut hi) = (0usize, 0usize);
        ws.rho.clear();
        ws.count.clear();
        for (k, &nk) in num.iter().enumerate() {
            while lo < onsets.len() && onsets[lo] < k {
                lo += 1;
            }
            while hi < onsets.len() && onsets[hi] < k + l {
                hi += 1;
            }
            let count = hi.saturating_sub(lo);
            let (s1, s2) = (p1[k + l] - p1[k], p2[k + l] - p2[k]);
            let energy = if self.template.centered { s2 - s1 * s1 / l as f64 } else { s2 };
            let valid = count >= self.min_strikes && energy > 1e-12 * s2.max(1e-30);
            ws.rho.push(if valid { (nk as f64 / energy.sqrt()) as f32 } else { f32::NAN });
            ws.count.push(count as u32);
        }
    }

    /// The `count` best lags, at least two smoothing widths apart.
    fn top_lags(&self, rho: &[f32], count: usize) -> Vec<usize> {
        let sep = ((2.0 * self.stage.smooth_s * self.stage.rate_hz) as usize).max(1);
        let mut chosen: Vec<usize> = Vec::with_capacity(count);
        for _ in 0..count {
            let pick = (0..rho.len())
                .filter(|&k| !rho[k].is_nan() && chosen.iter().all(|&c| c.abs_diff(k) >= sep))
                .max_by(|&a, &b| rho[a].total_cmp(&rho[b]));
            let Some(k) = pick else { break };
            chosen.push(k);
        }
        chosen
    }

    /// Replaces the correlation at the top few separated lags with the outlier-robust fit.
    fn rescore_peaks(&self, ws: &mut Workspace) {
        let l = self.template.len;
        for k in self.top_lags(&ws.rho, self.robust_lags) {
            let x = &ws.rendered.samples[k..k + l];
            ws.rho[k] = robust_rho(x, &self.template.values, &mut ws.robust_scratch) as f32;
        }
    }

    /// Replaces the correlation at the top lags with the per-strike loudness refit (see free_amp.rs).
    fn refit_peaks(&self, ws: &mut Workspace, fa: FreeAmp) {
        let exponent = self.phys.render_exponent();
        for k in self.top_lags(&ws.rho, fa.lags) {
            if let Some(r) = refit(&self.template.values, k, &ws.kernels, exponent, self.n_eff, fa) {
                ws.rho[k] = r.rho as f32;
                ws.penalty[k] = r.penalty;
            }
        }
    }
}

fn prefix_sums(m: &[f32], p1: &mut Vec<f64>, p2: &mut Vec<f64>) {
    p1.clear();
    p2.clear();
    p1.push(0.0);
    p2.push(0.0);
    let (mut a, mut b) = (0.0f64, 0.0f64);
    for &v in m {
        a += v as f64;
        b += (v as f64) * (v as f64);
        p1.push(a);
        p2.push(b);
    }
}

/// ln( (Σ exp(ll) + (n_total − len)) / n_total ): lags that cannot match count as LL = 0.
fn log_mean_exp_with_nulls(lls: &[f64], n_total: usize) -> f64 {
    if n_total == 0 {
        return 0.0;
    }
    let mx = lls.iter().copied().fold(0.0f64, f64::max);
    let nulls = (n_total - lls.len()) as f64;
    let sum: f64 = lls.iter().map(|v| (v - mx).exp()).sum::<f64>() + nulls * (-mx).exp();
    mx + sum.ln() - (n_total as f64).ln()
}
