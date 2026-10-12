use super::detect::Onset;
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::model::propagation::arrival;
use crate::search::grid;
use crate::strikes::{Strike, StrikeIndex};
use rayon::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct OnsetSolution {
    pub pos: LatLon,
    pub start: f64,
    pub start_utc: String,
    pub inliers: usize,
    pub onsets: usize,
    /// 1-sigma position uncertainty from the least-squares covariance.
    pub sigma_e_km: f64,
    pub sigma_n_km: f64,
    pub rms_resid_s: f64,
}

pub struct SolveParams {
    pub half_km: f64,
    pub spacing_km: f64,
    pub margin_s: f64,
    pub tol_s: f64,
    pub onset_sigma_s: f64,
}

/// Consensus voting over (cell, start time), then Gauss-Newton on the matched (onset, strike) pairs.
pub fn solve(
    index: &StrikeIndex,
    phys: &Physics,
    atm: &Atmosphere,
    onsets: &[Onset],
    center: LatLon,
    start_guess: f64,
    clip_s: f64,
    p: &SolveParams,
) -> Option<OnsetSolution> {
    let onsets = &onsets[..onsets.len().min(64)];
    let mut ids = Vec::new();
    index.within(&center, p.half_km + phys.audible_km, &mut ids);
    let (t_lo, t_hi) = (start_guess - p.margin_s - 150.0, start_guess + p.margin_s + clip_s);
    let local: Vec<&Strike> =
        ids.iter().map(|&i| &index.strikes[i as usize]).filter(|s| s.time >= t_lo && s.time <= t_hi).collect();

    let cells = grid::around(center, p.half_km, p.spacing_km);
    let t0 = start_guess - p.margin_s;
    let n_bins = (2.0 * p.margin_s / p.tol_s).ceil() as usize + 2;
    let (votes, ci, bi) = cells
        .par_iter()
        .enumerate()
        .map_init(
            || vec![0u64; n_bins],
            |masks, (ci, cell)| {
                masks.iter_mut().for_each(|m| *m = 0);
                for s in &local {
                    let a = arrival(ModelKind::Improved, phys, atm, s, &cell.center);
                    if !a.audible {
                        continue;
                    }
                    for (j, o) in onsets.iter().enumerate() {
                        let b = ((a.onset - o.t - t0) / p.tol_s).floor();
                        if b >= 0.0 && (b as usize) < n_bins {
                            masks[b as usize] |= 1 << j;
                        }
                    }
                }
                let (bi, v) = (0..n_bins - 1)
                    .map(|b| (b, (masks[b] | masks[b + 1]).count_ones()))
                    .max_by_key(|&(_, v)| v)
                    .unwrap();
                (v, ci, bi)
            },
        )
        .max_by_key(|&(v, _, _)| v)?;
    if votes < 3 {
        return None;
    }
    let start = t0 + (bi as f64 + 1.0) * p.tol_s;
    refine(&local, phys, atm, onsets, cells[ci].center, start, p)
}

/// Gauss-Newton on (east, north, start) with nearest-in-time re-assignment each iteration.
fn refine(local: &[&Strike], phys: &Physics, atm: &Atmosphere, onsets: &[Onset], seed: LatLon, start: f64, p: &SolveParams) -> Option<OnsetSolution> {
    let mut x = [0.0f64, 0.0, start];
    let onset_at = |s: &Strike, e: f64, n: f64| arrival(ModelKind::Improved, phys, atm, s, &seed.offset_km(e, n));
    let mut last = None;
    for _ in 0..12 {
        let mut jtj = [[0.0f64; 3]; 3];
        let mut jtr = [0.0f64; 3];
        let (mut ss, mut used) = (0.0, 0usize);
        for o in onsets {
            let target = x[2] + o.t;
            let best = local
                .iter()
                .map(|s| (s, onset_at(s, x[0], x[1])))
                .filter(|(_, a)| a.audible)
                .min_by(|a, b| (a.1.onset - target).abs().total_cmp(&(b.1.onset - target).abs()));
            let Some((s, a)) = best else { continue };
            let resid = target - a.onset;
            if resid.abs() > 2.0 * p.tol_s {
                continue;
            }
            const H: f64 = 0.01;
            let de = (onset_at(s, x[0] + H, x[1]).onset - a.onset) / H;
            let dn = (onset_at(s, x[0], x[1] + H).onset - a.onset) / H;
            let w = 1.0 / (p.onset_sigma_s.powi(2) + a.sigma_s.powi(2));
            // d(resid)/d(e, n, start)
            let j = [-de, -dn, 1.0];
            for r in 0..3 {
                jtr[r] += w * j[r] * resid;
                for c in 0..3 {
                    jtj[r][c] += w * j[r] * j[c];
                }
            }
            ss += resid * resid;
            used += 1;
        }
        if used < 4 {
            return None;
        }
        let step = solve3(jtj, jtr)?;
        for k in 0..3 {
            x[k] -= step[k];
        }
        let cov = invert3(jtj)?;
        let rms = (ss / used as f64).sqrt();
        // Scale covariance by the residual misfit when it exceeds the assumed timing noise.
        let scale = (rms / p.onset_sigma_s).max(1.0).powi(2);
        last = Some((used, rms, cov[0][0] * scale, cov[1][1] * scale));
        if step.iter().map(|v| v.abs()).fold(0.0, f64::max) < 1e-3 {
            break;
        }
    }
    let (inliers, rms, ve, vn) = last?;
    Some(OnsetSolution {
        pos: seed.offset_km(x[0], x[1]),
        start: x[2],
        start_utc: crate::geo::format_utc(x[2]),
        inliers,
        onsets: onsets.len(),
        sigma_e_km: ve.sqrt(),
        sigma_n_km: vn.sqrt(),
        rms_resid_s: rms,
    })
}

fn det3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn invert3(m: [[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let d = det3(&m);
    if d.abs() < 1e-18 {
        return None;
    }
    let mut inv = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            let (r1, r2) = ((c + 1) % 3, (c + 2) % 3);
            let (c1, c2) = ((r + 1) % 3, (r + 2) % 3);
            inv[r][c] = (m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1]) / d;
        }
    }
    Some(inv)
}

fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let inv = invert3(m)?;
    Some([0, 1, 2].map(|r| (0..3).map(|c| inv[r][c] * b[c]).sum()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invert_identity_scaled() {
        let m = [[2.0, 0.0, 0.0], [0.0, 4.0, 1.0], [0.0, 1.0, 3.0]];
        let inv = invert3(m).unwrap();
        for r in 0..3 {
            for c in 0..3 {
                let v: f64 = (0..3).map(|k| m[r][k] * inv[k][c]).sum();
                assert!((v - if r == c { 1.0 } else { 0.0 }).abs() < 1e-12);
            }
        }
    }
}
