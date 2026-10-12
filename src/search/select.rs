use super::grid::Cell;
use super::scorer::PointScore;
use crate::model::params::ModelKind;

/// Ranking key: the improved model ranks by evidence; the baseline by peak cosine, as in the video.
pub fn rank_key(kind: ModelKind, s: &PointScore) -> f64 {
    match kind {
        ModelKind::Improved => s.log_ev,
        ModelKind::Baseline => s.best_rho as f64,
    }
}

/// Indices of the best `k` cells, greedily skipping any within `min_sep_km` of one already chosen.
pub fn separated_peaks(cells: &[Cell], keys: &[f64], k: usize, min_sep_km: f64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..cells.len()).filter(|&i| keys[i].is_finite() && keys[i] > 0.0).collect();
    order.sort_by(|&a, &b| keys[b].total_cmp(&keys[a]));
    let mut picked: Vec<usize> = Vec::new();
    for i in order {
        if picked.len() == k {
            break;
        }
        if picked.iter().all(|&j| cells[i].center.dist_km(&cells[j].center) >= min_sep_km) {
            picked.push(i);
        }
    }
    picked
}

/// Posterior probability per cell from log-evidence under a uniform-per-area prior.
pub fn posterior(cells: &[Cell], log_ev: &[f64]) -> Vec<f64> {
    softmax(&cells.iter().zip(log_ev).map(|(c, l)| l + c.area_km2().ln()).collect::<Vec<_>>())
}

pub fn softmax(logs: &[f64]) -> Vec<f64> {
    let mx = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let w: Vec<f64> = logs.iter().map(|l| (l - mx).exp()).collect();
    let total: f64 = w.iter().sum();
    w.into_iter().map(|v| v / total).collect()
}

/// Mask of the smallest set of cells holding `mass` of the posterior, plus that set's area.
pub fn credible_region(cells: &[Cell], prob: &[f64], mass: f64) -> (Vec<bool>, f64) {
    let mut order: Vec<usize> = (0..prob.len()).collect();
    order.sort_by(|&a, &b| prob[b].total_cmp(&prob[a]));
    let mut inside = vec![false; prob.len()];
    let (mut acc, mut area) = (0.0, 0.0);
    for i in order {
        if acc >= mass {
            break;
        }
        inside[i] = true;
        acc += prob[i];
        area += cells[i].area_km2();
    }
    (inside, area)
}
