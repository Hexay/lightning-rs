use super::feature::Feature;
use std::time::Instant;
use super::ncc::{Scratch, Template};
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::model::propagation::{Arrival, arrival};
use crate::model::render::{Rendered, render};
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
    fn reduce(&self, origin: f64, ws: &Workspace) -> PointScore {
        let (num, p1, p2, onsets) = (&ws.ncc.num, &ws.p1, &ws.p2, &ws.rendered.onset_idx);
        let l = self.template.len;
        let (mut lo, mut hi) = (0usize, 0usize);
        let mut best = PointScore::default();
        let mut lls = Vec::new();
        for (k, &nk) in num.iter().enumerate() {
            while lo < onsets.len() && onsets[lo] < k {
                lo += 1;
            }
            while hi < onsets.len() && onsets[hi] < k + l {
                hi += 1;
            }
            let count = hi.saturating_sub(lo);
            if count < self.min_strikes {
                continue;
            }
            let (s1, s2) = (p1[k + l] - p1[k], p2[k + l] - p2[k]);
            let energy = if self.template.centered { s2 - s1 * s1 / l as f64 } else { s2 };
            if energy <= 1e-12 * s2.max(1e-30) {
                continue;
            }
            let rho = (nk as f64 / energy.sqrt()).clamp(0.0, 0.999);
            lls.push(-0.5 * self.n_eff * (1.0 - rho * rho).ln());
            if rho as f32 > best.best_rho {
                best.best_rho = rho as f32;
                best.best_start = origin + k as f64 / self.stage.rate_hz;
                best.strikes_in_window = count as u32;
            }
        }
        best.log_ev = log_mean_exp_with_nulls(&lls, num.len());
        best
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
