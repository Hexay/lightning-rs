use crate::geo::{LatLon, km_per_deg_lat, km_per_deg_lon};

/// A grid cell: centre plus half-extent in degrees (rows keep a constant km spacing in longitude).
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub center: LatLon,
    pub half_dlat: f64,
    pub half_dlon: f64,
}

impl Cell {
    pub fn area_km2(&self) -> f64 {
        4.0 * self.half_dlat * km_per_deg_lat() * self.half_dlon * km_per_deg_lon(self.center.lat)
    }
}

/// Equal-area-ish grid covering [lo, hi] at `spacing_km`.
pub fn cover(lo: LatLon, hi: LatLon, spacing_km: f64) -> Vec<Cell> {
    let dlat = spacing_km / km_per_deg_lat();
    let mut cells = Vec::new();
    let mut lat = lo.lat + dlat / 2.0;
    while lat < hi.lat + dlat / 2.0 {
        let dlon = spacing_km / km_per_deg_lon(lat);
        let mut lon = lo.lon + dlon / 2.0;
        while lon < hi.lon + dlon / 2.0 {
            cells.push(Cell { center: LatLon::new(lat, lon), half_dlat: dlat / 2.0, half_dlon: dlon / 2.0 });
            lon += dlon;
        }
        lat += dlat;
    }
    cells
}

/// Square grid of side 2·`half_km` centred on `c`.
pub fn around(c: LatLon, half_km: f64, spacing_km: f64) -> Vec<Cell> {
    cover(c.offset_km(-half_km, -half_km), c.offset_km(half_km, half_km), spacing_km)
}
