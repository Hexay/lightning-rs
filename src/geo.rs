pub const EARTH_RADIUS_KM: f64 = 6371.0;
const DEG: f64 = std::f64::consts::PI / 180.0;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl LatLon {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// East/north offset in km from `self` to `other` (local tangent plane; fine below ~100 km).
    pub fn enu_km(&self, other: &LatLon) -> (f64, f64) {
        let mean_lat = 0.5 * (self.lat + other.lat) * DEG;
        let east = (other.lon - self.lon) * DEG * mean_lat.cos() * EARTH_RADIUS_KM;
        let north = (other.lat - self.lat) * DEG * EARTH_RADIUS_KM;
        (east, north)
    }

    pub fn dist_km(&self, other: &LatLon) -> f64 {
        let (e, n) = self.enu_km(other);
        e.hypot(n)
    }

    pub fn offset_km(&self, east: f64, north: f64) -> LatLon {
        let lat = self.lat + north / EARTH_RADIUS_KM / DEG;
        let lon = self.lon + east / (EARTH_RADIUS_KM * (self.lat * DEG).cos()) / DEG;
        LatLon { lat, lon }
    }
}

pub fn km_per_deg_lat() -> f64 {
    EARTH_RADIUS_KM * DEG
}

pub fn km_per_deg_lon(lat: f64) -> f64 {
    EARTH_RADIUS_KM * DEG * (lat * DEG).cos()
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Inverse of `days_from_civil`.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

/// Parses `YYYY-MM-DD` into epoch seconds at 00:00 UTC.
pub fn parse_date(s: &str) -> anyhow::Result<f64> {
    let parts: Vec<&str> = s.split('-').collect();
    anyhow::ensure!(parts.len() == 3, "expected YYYY-MM-DD, got {s}");
    let days = days_from_civil(parts[0].parse()?, parts[1].parse()?, parts[2].parse()?);
    Ok(days as f64 * 86400.0)
}

/// `HH:MM:SS[.sss]` on the UTC day starting at epoch `day`.
pub fn parse_hms(day: f64, s: &str) -> anyhow::Result<f64> {
    let hms: Vec<f64> = s.split(':').map(str::parse).collect::<Result<_, _>>()?;
    anyhow::ensure!(hms.len() == 3, "time must be HH:MM:SS[.sss], got {s}");
    Ok(day + hms[0] * 3600.0 + hms[1] * 60.0 + hms[2])
}

pub fn format_utc(epoch: f64) -> String {
    let secs = epoch.floor() as i64;
    let (y, m, d) = civil_from_days(secs.div_euclid(86400));
    let sod = secs.rem_euclid(86400);
    let frac = epoch - secs as f64;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:04.1}Z",
        sod / 3600,
        (sod / 60) % 60,
        (sod % 60) as f64 + frac
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_roundtrip() {
        let d = days_from_civil(2025, 7, 19);
        assert_eq!(civil_from_days(d), (2025, 7, 19));
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn offset_inverts_enu() {
        let a = LatLon::new(65.75, 14.0);
        let b = a.offset_km(3.0, -4.0);
        let (e, n) = a.enu_km(&b);
        assert!((e - 3.0).abs() < 0.01 && (n + 4.0).abs() < 0.01);
    }
}
