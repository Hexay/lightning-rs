pub mod csv;
pub mod frost;
pub mod ualf;

use crate::geo::{LatLon, km_per_deg_lat, km_per_deg_lon};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Frost day cache: `$THUNDERPRINT_CACHE` if set, else `data/frost` under the working directory.
pub fn cache_dir() -> PathBuf {
    std::env::var_os("THUNDERPRINT_CACHE").map_or_else(|| PathBuf::from("data/frost"), PathBuf::from)
}

/// Strikes in [start, end) from a user CSV if given, else from Frost (downloaded and cached).
pub fn load_range(start: f64, end: f64, csv: Option<&Path>) -> anyhow::Result<Vec<Strike>> {
    match csv {
        Some(p) => Ok(csv::load(p)?.into_iter().filter(|s| s.time >= start && s.time < end).collect()),
        None => frost::load_range(start, end, &cache_dir()),
    }
}

#[derive(Clone, Debug)]
pub struct Strike {
    /// Epoch seconds (UTC).
    pub time: f64,
    pub pos: LatLon,
    pub peak_ka: f32,
    /// Error ellipse orientation, degrees clockwise from north.
    pub ellipse_angle_deg: f32,
    pub semi_major_km: f32,
    pub semi_minor_km: f32,
    /// In-cloud (true) vs cloud-to-ground (false).
    pub cloud: bool,
}

impl Strike {
    /// 1-sigma position error (km) projected onto a horizontal unit direction (east, north).
    pub fn sigma_along_km(&self, dir_e: f64, dir_n: f64) -> f64 {
        // UALF ellipses are 50% confidence; 2-D Gaussian 50% radius = 1.1774 sigma.
        const ELLIPSE50_TO_SIGMA: f64 = 1.0 / 1.1774;
        let a = self.semi_major_km as f64 * ELLIPSE50_TO_SIGMA;
        let b = self.semi_minor_km as f64 * ELLIPSE50_TO_SIGMA;
        let th = (self.ellipse_angle_deg as f64).to_radians();
        let (maj_e, maj_n) = (th.sin(), th.cos());
        let along = dir_e * maj_e + dir_n * maj_n;
        (a * a * along * along + b * b * (1.0 - along * along)).sqrt()
    }
}

const CELL_DEG_LAT: f64 = 0.2;
const CELL_DEG_LON: f64 = 0.4;

/// Strikes bucketed into lat/lon cells for radius queries.
pub struct StrikeIndex {
    pub strikes: Vec<Strike>,
    cells: HashMap<(i32, i32), Vec<u32>>,
}

impl StrikeIndex {
    pub fn new(mut strikes: Vec<Strike>) -> Self {
        strikes.sort_by(|a, b| a.time.total_cmp(&b.time));
        let mut cells: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (i, s) in strikes.iter().enumerate() {
            cells.entry(cell_of(&s.pos)).or_default().push(i as u32);
        }
        Self { strikes, cells }
    }

    /// Indices of strikes within `radius_km` of `p`, ascending (= time order).
    pub fn within(&self, p: &LatLon, radius_km: f64, out: &mut Vec<u32>) {
        out.clear();
        let dlat = radius_km / km_per_deg_lat();
        let dlon = radius_km / km_per_deg_lon(p.lat.abs().min(89.0) + dlat);
        let lo = cell_of(&LatLon::new(p.lat - dlat, p.lon - dlon));
        let hi = cell_of(&LatLon::new(p.lat + dlat, p.lon + dlon));
        for ci in lo.0..=hi.0 {
            for cj in lo.1..=hi.1 {
                let Some(ids) = self.cells.get(&(ci, cj)) else { continue };
                out.extend(ids.iter().copied().filter(|&i| p.dist_km(&self.strikes[i as usize].pos) <= radius_km));
            }
        }
        out.sort_unstable();
    }

    pub fn bbox(&self) -> Option<(LatLon, LatLon)> {
        let first = self.strikes.first()?.pos;
        Some(self.strikes.iter().fold((first, first), |(lo, hi), s| {
            (
                LatLon::new(lo.lat.min(s.pos.lat), lo.lon.min(s.pos.lon)),
                LatLon::new(hi.lat.max(s.pos.lat), hi.lon.max(s.pos.lon)),
            )
        }))
    }
}

fn cell_of(p: &LatLon) -> (i32, i32) {
    ((p.lat / CELL_DEG_LAT).floor() as i32, (p.lon / CELL_DEG_LON).floor() as i32)
}
