use super::pipeline::{Region, SearchConfig, observed_feature, scorer};
use super::scorer::Workspace;
use crate::audio::BandAudio;
use crate::geo::LatLon;
use crate::model::params::Atmosphere;
use crate::strikes::StrikeIndex;
use serde::Serialize;

/// Observed vs modelled fine-stage envelopes, each scaled to unit max, for the viewer's trace panel.
#[derive(Serialize)]
pub struct Traces {
    pub rate_hz: f64,
    pub observed: Vec<f32>,
    pub regions: Vec<RegionTrace>,
}

#[derive(Serialize)]
pub struct RegionTrace {
    /// 1-based, as displayed (matches the GeoJSON candidate rank).
    pub rank: usize,
    pub start: f64,
    pub model: Vec<f32>,
}

pub fn region_traces(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, regions: &[Region]) -> Traces {
    let regions = regions
        .iter()
        .map(|r| RegionTrace {
            rank: r.rank + 1,
            start: r.score.best_start,
            model: fitted_pair(cfg, index, audio, r.map, &r.atmosphere, r.score.best_start).1,
        })
        .collect();
    Traces { rate_hz: cfg.fine.rate_hz, observed: unit_max(observed_feature(cfg, audio, cfg.fine)), regions }
}

/// (observed, model) at the fine stage for a listener at `p` whose recording starts at `start`.
pub fn fitted_pair(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, p: LatLon, atm: &Atmosphere, start: f64) -> (Vec<f32>, Vec<f32>) {
    let st = cfg.fine;
    let sc = scorer(cfg, index, audio, st, (start, start + 0.5 / st.rate_hz));
    let model = sc.model_window(&p, atm, &mut Workspace::default());
    (unit_max(observed_feature(cfg, audio, st)), unit_max(model))
}

fn unit_max(mut v: Vec<f32>) -> Vec<f32> {
    let m = v.iter().fold(1e-30f32, |m, x| m.max(*x));
    v.iter_mut().for_each(|x| *x = (*x / m * 1e3).round() / 1e3);
    v
}
