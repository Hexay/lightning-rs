use super::feature::Feature;
use super::free_amp::FreeAmp;
use super::grid::{self, Cell};
use super::ncc::Template;
use super::scorer::{PointScore, Scorer, Stage, Workspace};
use super::select::{credible_region, posterior, rank_key, separated_peaks, softmax};
use crate::audio::{BandAudio, observed_envelope};
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, BASELINE_C_MPS, ModelKind, Physics};
use crate::strikes::StrikeIndex;
use rayon::prelude::*;
use serde::Serialize;
use std::time::Instant;

pub struct SearchConfig {
    pub kind: ModelKind,
    pub phys: Physics,
    pub coarse: Stage,
    pub mid: Stage,
    pub fine: Stage,
    pub min_strikes: usize,
    pub coarse_peaks: usize,
    pub fine_regions: usize,
    pub mid_half_km: f64,
    pub fine_half_km: f64,
    pub fine_time_margin_s: f64,
    pub polish_half_km: f64,
    pub polish_spacing_km: f64,
    /// Atmospheres marginalised over in the fine stage (improved model only).
    pub nuisance: Vec<Atmosphere>,
    pub floor_window_s: f64,
    /// Fraction of nominal independent samples credited to the likelihood (tempering for model error).
    pub n_eff_factor: f64,
    pub feature: Feature,
    pub robust_lags: usize,
    /// Fine-stage per-strike loudness refit (improved model, envelope feature).
    pub free_amp: Option<FreeAmp>,
}

#[derive(Serialize)]
pub struct ScoredCell {
    #[serde(skip)]
    pub cell: Cell,
    pub level: &'static str,
    pub region: Option<usize>,
    pub score: PointScore,
    pub prob: f64,
    pub in_cr90: bool,
}

#[derive(Serialize)]
pub struct Region {
    pub rank: usize,
    pub map: LatLon,
    pub score: PointScore,
    pub start_utc: String,
    pub atmosphere: Atmosphere,
    pub cr90_area_km2: f64,
    pub region_prob: f64,
}

pub struct SearchResult {
    pub cells: Vec<ScoredCell>,
    pub regions: Vec<Region>,
}

pub fn run(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, day: (f64, f64)) -> SearchResult {
    let default_atm = match cfg.kind {
        ModelKind::Improved => Atmosphere::from_temp(15.0, 0.0, 0.0),
        ModelKind::Baseline => Atmosphere::still(BASELINE_C_MPS),
    };
    let day_range = (day.0, day.1 - audio.duration_s);
    let mut out = Vec::new();

    let (lo, hi) = index.bbox().expect("no strikes");
    let coarse_scorer = scorer(cfg, index, audio, cfg.coarse, day_range);
    let all = grid::cover(lo, hi, cfg.coarse.spacing_km);
    let coarse_cells = prune_inactive(&all, index, cfg, audio.duration_s);
    let coarse = score_cells(&coarse_scorer, &coarse_cells, &[default_atm], "coarse");
    // Coarse/mid evidence is swamped by the day-long time prior (≈0 everywhere); peak correlation selects.
    let keys: Vec<f64> = coarse.iter().map(|s| s.best_rho as f64).collect();
    // No separation here: near-duplicates of a strong peak must not crowd out the truth's own cell.
    let peaks = separated_peaks(&coarse_cells, &keys, cfg.coarse_peaks, 0.0);
    push_level(&mut out, &coarse_cells, coarse, "coarse", None);

    let mid_scorer = scorer(cfg, index, audio, cfg.mid, day_range);
    let mut mid_cells = Vec::new();
    for &p in &peaks {
        mid_cells.extend(grid::around(coarse_cells[p].center, cfg.mid_half_km, cfg.mid.spacing_km));
    }
    let mid = score_cells(&mid_scorer, &mid_cells, &[default_atm], "mid");
    let keys: Vec<f64> = mid.iter().map(|s| s.best_rho as f64).collect();
    let fine_peaks = separated_peaks(&mid_cells, &keys, cfg.fine_regions, 2.0 * cfg.fine_half_km);
    let fine_seeds: Vec<(LatLon, f64)> = fine_peaks.iter().map(|&i| (mid_cells[i].center, mid[i].best_start)).collect();
    push_level(&mut out, &mid_cells, mid, "mid", None);

    let nuisance = match cfg.kind {
        ModelKind::Improved if !cfg.nuisance.is_empty() => cfg.nuisance.clone(),
        _ => vec![default_atm],
    };
    let mut regions = Vec::new();
    for (r, (center, start)) in fine_seeds.into_iter().enumerate() {
        let range = (start - cfg.fine_time_margin_s, start + cfg.fine_time_margin_s);
        let mut fine_scorer = scorer(cfg, index, audio, cfg.fine, range);
        if cfg.kind == ModelKind::Improved && cfg.feature() == Feature::Envelope {
            fine_scorer.free_amp = cfg.free_amp;
        }
        let cells = grid::around(center, cfg.fine_half_km, cfg.fine.spacing_km);
        let (scores, atms) = score_cells_marginal(&fine_scorer, &cells, &nuisance);
        let first = out.len();
        push_level(&mut out, &cells, scores, "fine", Some(r));
        let region_cells = &mut out[first..];
        let map_i = (0..region_cells.len())
            .max_by(|&a, &b| rank_key(cfg.kind, &region_cells[a].score).total_cmp(&rank_key(cfg.kind, &region_cells[b].score)))
            .unwrap();
        let (inside, area) = credible_region(&cells, &region_cells.iter().map(|c| c.prob).collect::<Vec<_>>(), 0.9);
        region_cells.iter_mut().zip(inside).for_each(|(c, i)| c.in_cr90 = i);
        let (map, best) = polish(cfg, &fine_scorer, cells[map_i].center, atms[map_i]);
        regions.push(Region {
            rank: r,
            map,
            score: best,
            start_utc: crate::geo::format_utc(best.best_start),
            atmosphere: atms[map_i],
            cr90_area_km2: area,
            region_prob: 0.0,
        });
    }
    regions.sort_by(|a, b| rank_key(cfg.kind, &b.score).total_cmp(&rank_key(cfg.kind, &a.score)));
    let region_post = posterior_of_regions(&regions);
    for (i, (reg, p)) in regions.iter_mut().zip(region_post).enumerate() {
        reg.rank = i;
        reg.region_prob = p;
    }
    SearchResult { cells: out, regions }
}

pub(crate) fn scorer<'a>(cfg: &'a SearchConfig, index: &'a StrikeIndex, audio: &BandAudio, stage: Stage, range: (f64, f64)) -> Scorer<'a> {
    let n_eff = audio.duration_s / (2.0 * std::f64::consts::PI.sqrt() * stage.smooth_s.max(0.5 / stage.rate_hz));
    Scorer {
        index,
        kind: cfg.kind,
        phys: &cfg.phys,
        stage,
        feature: cfg.feature(),
        template: Template::new(&observed_feature(cfg, audio, stage), cfg.kind == ModelKind::Improved),
        start_range: range,
        min_strikes: cfg.min_strikes,
        robust_lags: if cfg.kind == ModelKind::Improved { cfg.robust_lags } else { 0 },
        free_amp: None,
        n_eff: n_eff * cfg.n_eff_factor,
    }
}

/// The observation exactly as the scorer correlates it: envelope, floor removal, compression, feature.
pub(crate) fn observed_feature(cfg: &SearchConfig, audio: &BandAudio, stage: Stage) -> Vec<f32> {
    let improved = cfg.kind == ModelKind::Improved;
    let gamma = if improved { cfg.phys.gamma as f32 } else { 1.0 };
    let mut obs: Vec<f32> = observed_envelope(audio, stage.rate_hz, stage.smooth_s, improved.then_some(cfg.floor_window_s))
        .iter()
        .map(|v| v.max(0.0).powf(gamma))
        .collect();
    cfg.feature().apply(&mut obs, stage.rate_hz, stage.smooth_s);
    obs
}

impl SearchConfig {
    /// The baseline always correlates the raw envelope, as in the video.
    pub fn feature(&self) -> Feature {
        if self.kind == ModelKind::Improved { self.feature } else { Feature::Envelope }
    }
}

/// Sub-grid refinement of a region's best cell under that cell's best atmosphere.
fn polish(cfg: &SearchConfig, scorer: &Scorer, center: LatLon, atm: Atmosphere) -> (LatLon, PointScore) {
    let cells = grid::around(center, cfg.polish_half_km, cfg.polish_spacing_km);
    let scores = score_cells_marginal(scorer, &cells, &[atm]).0;
    let i = (0..cells.len()).max_by(|&a, &b| rank_key(cfg.kind, &scores[a]).total_cmp(&rank_key(cfg.kind, &scores[b]))).unwrap();
    (cells[i].center, scores[i])
}

/// Drops cells that never see `min_strikes` audible strikes within one clip-length window.
fn prune_inactive(cells: &[Cell], index: &StrikeIndex, cfg: &SearchConfig, clip_s: f64) -> Vec<Cell> {
    let radius = match cfg.kind {
        ModelKind::Improved => cfg.phys.audible_km,
        ModelKind::Baseline => crate::model::params::BASELINE_RADIUS_KM,
    };
    let window = clip_s + radius / 0.33;
    cells
        .par_iter()
        .map_init(Vec::new, |ids, c| {
            index.within(&c.center, radius, ids);
            let times: Vec<f64> = ids.iter().map(|&i| index.strikes[i as usize].time).collect();
            let mut lo = 0;
            let active = (0..times.len()).any(|hi| {
                while times[hi] - times[lo] > window {
                    lo += 1;
                }
                hi + 1 - lo >= cfg.min_strikes
            });
            active.then_some(*c)
        })
        .flatten()
        .collect()
}

fn score_cells(scorer: &Scorer, cells: &[Cell], atm: &[Atmosphere], label: &str) -> Vec<PointScore> {
    let t = Instant::now();
    let scores = score_cells_marginal(scorer, cells, atm).0;
    let best = scores.iter().map(|s| s.best_rho).fold(0.0f32, f32::max);
    eprintln!("  {label}: {} cells in {:.1}s, best rho {best:.3}", cells.len(), t.elapsed().as_secs_f64());
    scores
}

/// Scores each cell under every atmosphere; evidence is averaged (marginalised), the best one reported.
fn score_cells_marginal(scorer: &Scorer, cells: &[Cell], atms: &[Atmosphere]) -> (Vec<PointScore>, Vec<Atmosphere>) {
    let t = Instant::now();
    let res: Vec<(PointScore, Atmosphere)> = cells
        .par_iter()
        .map_init(Workspace::default, |ws, c| {
            let per: Vec<PointScore> = atms.iter().map(|a| scorer.score(&c.center, a, ws)).collect();
            let best = (0..per.len()).max_by(|&a, &b| per[a].log_ev.total_cmp(&per[b].log_ev)).unwrap();
            let lls: Vec<f64> = per.iter().map(|p| p.log_ev).collect();
            (PointScore { log_ev: log_mean_exp(&lls), ..per[best] }, atms[best])
        })
        .collect();
    if atms.len() > 1 {
        eprintln!("  fine: {} cells x {} atmospheres in {:.1}s", cells.len(), atms.len(), t.elapsed().as_secs_f64());
    }
    res.into_iter().unzip()
}

fn log_mean_exp(v: &[f64]) -> f64 {
    let mx = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    mx + (v.iter().map(|x| (x - mx).exp()).sum::<f64>() / v.len() as f64).ln()
}

fn push_level(out: &mut Vec<ScoredCell>, cells: &[Cell], scores: Vec<PointScore>, level: &'static str, region: Option<usize>) {
    let prob = posterior(cells, &scores.iter().map(|s| s.log_ev).collect::<Vec<_>>());
    out.extend(cells.iter().zip(scores).zip(prob).map(|((c, s), p)| ScoredCell {
        cell: *c,
        level,
        region,
        score: s,
        prob: p,
        in_cr90: false,
    }));
}

fn posterior_of_regions(regions: &[Region]) -> Vec<f64> {
    softmax(&regions.iter().map(|r| r.score.log_ev).collect::<Vec<_>>())
}
