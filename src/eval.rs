use crate::geo::LatLon;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

/// Ground truth, read only by evaluation (never by the search).
#[derive(Deserialize)]
pub struct Truth {
    pub lat: f64,
    pub lon: f64,
    /// Optional known recording start, epoch seconds.
    pub start: Option<f64>,
}

pub fn load_truth(path: &Path) -> Result<Truth> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(toml::from_str(&text)?)
}

/// Prints error of each candidate and whether the truth sits inside its fine 90% region.
pub fn report(run_dir: &Path, truth: &Truth) -> Result<()> {
    let t = LatLon::new(truth.lat, truth.lon);
    let cands: Value = serde_json::from_str(&std::fs::read_to_string(run_dir.join("candidates.json"))?)?;
    let heat: Value = serde_json::from_str(&std::fs::read_to_string(run_dir.join("heatmap.geojson"))?)?;
    println!("run: {}", cands["run"]);
    for r in cands["regions"].as_array().context("no regions")? {
        let p = LatLon::new(r["map"]["lat"].as_f64().unwrap(), r["map"]["lon"].as_f64().unwrap());
        let mut line = format!(
            "#{} error {:.2} km  P(region) {:.1}%  rho {:.3}  cr90 {:.2} km²",
            r["rank"].as_u64().unwrap() + 1,
            p.dist_km(&t),
            r["region_prob"].as_f64().unwrap() * 100.0,
            r["score"]["best_rho"].as_f64().unwrap(),
            r["cr90_area_km2"].as_f64().unwrap(),
        );
        if let Some(s) = truth.start {
            line += &format!("  start error {:+.1} s", r["score"]["best_start"].as_f64().unwrap() - s);
        }
        println!("{line}");
    }
    for s in cands["onset_solutions"].as_array().into_iter().flatten() {
        let p = LatLon::new(s["pos"]["lat"].as_f64().unwrap(), s["pos"]["lon"].as_f64().unwrap());
        println!(
            "onset solve error {:.2} km (1σ {:.2}/{:.2} km, {} matched)",
            p.dist_km(&t),
            s["sigma_e_km"].as_f64().unwrap(),
            s["sigma_n_km"].as_f64().unwrap(),
            s["inliers"]
        );
    }
    let inside = heat["features"].as_array().unwrap().iter().any(|f| {
        let pr = &f["properties"];
        pr["level"] == "fine" && pr["in_cr90"] == true && contains(&f["geometry"], &t)
    });
    println!("truth inside a fine 90% region: {inside}");
    for level in ["coarse", "mid"] {
        println!("{level}: {}", truth_rank(&heat, level, &t));
    }
    Ok(())
}

/// Rank (by peak correlation) of the cell containing the truth within one level.
fn truth_rank(heat: &Value, level: &str, t: &LatLon) -> String {
    let cells: Vec<&Value> = heat["features"].as_array().unwrap().iter().filter(|f| f["properties"]["level"] == level).collect();
    let rho = |f: &Value| f["properties"]["rho"].as_f64().unwrap_or(0.0);
    let Some(hit) = cells.iter().find(|f| contains(&f["geometry"], t)) else {
        return format!("truth cell not scored ({} cells)", cells.len());
    };
    let r = rho(hit);
    let rank = cells.iter().filter(|f| rho(f) > r).count() + 1;
    let best = cells.iter().map(|f| rho(f)).fold(0.0, f64::max);
    format!("truth cell rho {r:.3} rank {rank}/{} (best {best:.3})", cells.len())
}

fn contains(geom: &Value, p: &LatLon) -> bool {
    let ring = &geom["coordinates"][0];
    let (x0, y0) = (ring[0][0].as_f64().unwrap(), ring[0][1].as_f64().unwrap());
    let (x1, y1) = (ring[2][0].as_f64().unwrap(), ring[2][1].as_f64().unwrap());
    (x0..=x1).contains(&p.lon) && (y0..=y1).contains(&p.lat)
}
