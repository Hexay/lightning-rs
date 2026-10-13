use crate::onset::solve::OnsetSolution;
use crate::search::pipeline::{Region, ScoredCell};
use serde_json::{Value, json};

/// Cells with positive evidence as polygons, plus one point per candidate region.
pub fn feature_collection(cells: &[ScoredCell], regions: &[Region], onsets: &[OnsetSolution]) -> Value {
    let mut features: Vec<Value> = cells
        .iter()
        .filter(|c| c.score.log_ev > 0.0 || c.level == "fine")
        .map(|c| {
            let (p, a, b) = (c.cell.center, c.cell.half_dlat, c.cell.half_dlon);
            let ring = [(-b, -a), (b, -a), (b, a), (-b, a), (-b, -a)].map(|(dx, dy)| [p.lon + dx, p.lat + dy]);
            json!({
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [ring] },
                "properties": {
                    "kind": "cell",
                    "level": c.level,
                    "region": c.region,
                    "prob": c.prob,
                    "log_ev": c.score.log_ev,
                    "rho": c.score.best_rho,
                    "start_utc": crate::geo::format_utc(c.score.best_start),
                    "strikes": c.score.strikes_in_window,
                    "in_cr90": c.in_cr90,
                }
            })
        })
        .collect();
    features.extend(regions.iter().map(|r| {
        json!({
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [r.map.lon, r.map.lat] },
            "properties": {
                "kind": "candidate",
                "rank": r.rank + 1,
                "region_prob": r.region_prob,
                "log_ev": r.score.log_ev,
                "rho": r.score.best_rho,
                "start_utc": r.start_utc,
                "c_mps": r.atmosphere.c_mps,
                "wind_e_mps": r.atmosphere.wind_e_mps,
                "wind_n_mps": r.atmosphere.wind_n_mps,
                "cr90_area_km2": r.cr90_area_km2,
                "shape": r.shape,
                "reliable": r.shape.reliable(),
            }
        })
    }));
    features.extend(regions.iter().map(|r| {
        let ring: Vec<[f64; 2]> = r.shape.ellipse_ring(48).iter().map(|p| [p.lon, p.lat]).collect();
        json!({
            "type": "Feature",
            "geometry": { "type": "Polygon", "coordinates": [ring] },
            "properties": { "kind": "ellipse", "rank": r.rank + 1 }
        })
    }));
    features.extend(onsets.iter().map(|s| {
        json!({
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [s.pos.lon, s.pos.lat] },
            "properties": {
                "kind": "onset",
                "start_utc": s.start_utc,
                "inliers": s.inliers,
                "onsets": s.onsets,
                "sigma_e_km": s.sigma_e_km,
                "sigma_n_km": s.sigma_n_km,
                "rms_resid_s": s.rms_resid_s,
            }
        })
    }));
    json!({ "type": "FeatureCollection", "features": features })
}
