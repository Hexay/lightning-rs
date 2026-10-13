//! Per-region diagnostics: the posterior's error ellipse and how well the heard strikes surround the
//! listener. A one-sided storm leaves distance-to-storm nearly unconstrained (start time trades off
//! against it), so the ellipse's long axis points at the storm; few strikes or a narrow fan of azimuths
//! warn that the fix is weak.

use super::grid::Cell;
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::model::propagation::arrival;
use crate::strikes::StrikeIndex;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Shape {
    /// Posterior mean: the 90% ellipse's centre.
    pub center: LatLon,
    /// 90% ellipse semi-axes (Gaussian approximation of the fine posterior).
    pub major_km: f64,
    pub minor_km: f64,
    pub major_bearing_deg: f64,
    /// Loudness-weighted bearing from the listener to the strikes heard in the clip.
    pub storm_bearing_deg: f64,
    pub audible_strikes: usize,
    /// Circular std-dev of heard-strike bearings (loudness-weighted); small = one-sided storm.
    pub azimuth_spread_deg: f64,
}

/// Fewer heard strikes than this and a pick is likely a false match: on the sim suite and real clip,
/// correct picks heard ≥ 76 strikes and every wrong one ≤ 21, while fit and posterior did not separate them.
pub const MIN_RELIABLE_STRIKES: usize = 30;

/// √χ²₂(0.9): scales 1-σ axes to the 90% ellipse.
const ELLIPSE_90: f64 = 2.146;

pub fn shape(cells: &[Cell], prob: &[f64], heard: Heard) -> Shape {
    let (center, major_km, minor_km, major_bearing_deg) = ellipse(cells, prob, heard.listener);
    let (storm_bearing_deg, azimuth_spread_deg, audible_strikes) = heard.bearings();
    Shape { center, major_km, minor_km, major_bearing_deg, storm_bearing_deg, audible_strikes, azimuth_spread_deg }
}

impl Shape {
    pub fn reliable(&self) -> bool {
        self.audible_strikes >= MIN_RELIABLE_STRIKES
    }

    /// The 90% ellipse as a closed lat/lon ring of `n` points.
    pub fn ellipse_ring(&self, n: usize) -> Vec<LatLon> {
        let b = self.major_bearing_deg.to_radians();
        let (ue, un) = (b.sin(), b.cos());
        (0..=n)
            .map(|k| {
                let t = std::f64::consts::TAU * k as f64 / n as f64;
                let (a, c) = (self.major_km * t.cos(), self.minor_km * t.sin());
                self.center.offset_km(a * ue + c * un, a * un - c * ue)
            })
            .collect()
    }
}

fn ellipse(cells: &[Cell], prob: &[f64], origin: LatLon) -> (LatLon, f64, f64, f64) {
    let total: f64 = prob.iter().sum();
    if total <= 0.0 {
        return (origin, 0.0, 0.0, 0.0);
    }
    let pts: Vec<(f64, f64, f64)> = cells.iter().zip(prob).map(|(c, &p)| {
        let (e, n) = origin.enu_km(&c.center);
        (e, n, p / total)
    }).collect();
    let (me, mn) = pts.iter().fold((0.0, 0.0), |(a, b), &(e, n, w)| (a + w * e, b + w * n));
    let (mut cee, mut cnn, mut cen) = (0.0, 0.0, 0.0);
    for &(e, n, w) in &pts {
        let (de, dn) = (e - me, n - mn);
        cee += w * de * de;
        cnn += w * dn * dn;
        cen += w * de * dn;
    }
    let tr = 0.5 * (cee + cnn);
    let disc = (0.25 * (cee - cnn).powi(2) + cen * cen).sqrt();
    let (l1, l2) = (tr + disc, (tr - disc).max(0.0));
    let theta = 0.5 * (2.0 * cen).atan2(cee - cnn);
    // theta is measured from east; convert to a compass bearing in [0, 180).
    let bearing = (90.0 - theta.to_degrees()).rem_euclid(180.0);
    (origin.offset_km(me, mn), ELLIPSE_90 * l1.sqrt(), ELLIPSE_90 * l2.sqrt(), bearing)
}

/// What a listener at `listener` hears in the clip window under the model.
pub struct Heard<'a> {
    pub index: &'a StrikeIndex,
    pub phys: &'a Physics,
    pub atm: Atmosphere,
    pub listener: LatLon,
    pub start: f64,
    pub duration_s: f64,
}

impl Heard<'_> {
    /// (loudness-weighted mean bearing, circular spread, count) of audible strikes arriving in the clip.
    fn bearings(&self) -> (f64, f64, usize) {
        let mut ids = Vec::new();
        self.index.within(&self.listener, self.phys.audible_km, &mut ids);
        let (mut se, mut sn, mut sw, mut count) = (0.0, 0.0, 0.0, 0);
        for &i in &ids {
            let s = &self.index.strikes[i as usize];
            let a = arrival(ModelKind::Improved, self.phys, &self.atm, s, &self.listener);
            if !a.audible || a.onset < self.start || a.onset > self.start + self.duration_s {
                continue;
            }
            let (e, n) = self.listener.enu_km(&s.pos);
            let d = e.hypot(n).max(1e-6);
            let w = a.area.sqrt();
            se += w * e / d;
            sn += w * n / d;
            sw += w;
            count += 1;
        }
        if sw <= 0.0 {
            return (0.0, 180.0, 0);
        }
        let r = (se.hypot(sn) / sw).clamp(1e-9, 1.0);
        let bearing = se.atan2(sn).to_degrees().rem_euclid(360.0);
        (bearing, (-2.0 * r.ln()).sqrt().to_degrees(), count)
    }
}
