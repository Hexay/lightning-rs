use super::Strike;
use crate::geo::{LatLon, parse_iso};
use anyhow::{Context, Result, bail};
use std::path::Path;

/// Defaults for columns other networks often lack (e.g. Blitzortung has no current or CG/IC flag).
const DEFAULT_PEAK_KA: f32 = 10.0;
const DEFAULT_ERROR_KM: f32 = 0.5;

/// Loads a header-named CSV of strikes. Required: `time` (epoch seconds or ISO-8601 UTC), `lat`, `lon`.
/// Optional: `peak_ka`, `cloud` (1/true = in-cloud), `error_km` (50% location error radius).
pub fn load(path: &Path) -> Result<Vec<Strike>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = lines.next().context("empty strikes CSV")?.split(',').map(|h| h.trim().to_lowercase()).collect();
    let col = |name: &str| header.iter().position(|h| h == name);
    let (Some(t), Some(lat), Some(lon)) = (col("time"), col("lat"), col("lon")) else {
        bail!("strikes CSV needs time, lat, lon columns; got {header:?}");
    };
    let (ka, cloud, err) = (col("peak_ka"), col("cloud"), col("error_km"));
    lines
        .enumerate()
        .map(|(i, line)| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let num = |c: usize| -> Result<f64> { f.get(c).context("missing column")?.parse::<f64>().map_err(Into::into) };
            let opt = |c: Option<usize>, default: f32| -> Result<f32> { c.map_or(Ok(default), |c| Ok(num(c)? as f32)) };
            let row = || -> Result<Strike> {
                let raw_t = f.get(t).context("missing time")?;
                let error_km = opt(err, DEFAULT_ERROR_KM)?;
                Ok(Strike {
                    time: raw_t.parse::<f64>().or_else(|_| parse_iso(raw_t))?,
                    pos: LatLon::new(num(lat)?, num(lon)?),
                    peak_ka: opt(ka, DEFAULT_PEAK_KA)?,
                    ellipse_angle_deg: 0.0,
                    semi_major_km: error_km,
                    semi_minor_km: error_km,
                    cloud: cloud.and_then(|c| f.get(c)).is_some_and(|v| matches!(*v, "1" | "true" | "True" | "TRUE")),
                })
            };
            row().with_context(|| format!("{} line {}", path.display(), i + 2))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_and_full_rows() {
        let dir = std::env::temp_dir().join("lightning_rs_csv_test.csv");
        std::fs::write(&dir, "time,lat,lon,peak_ka,cloud\n1752940800.5,65.1,14.2,-23.4,0\n2025-07-19T16:00:01.25Z,65.2,14.3,8,1\n").unwrap();
        let s = load(&dir).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].time, 1752940800.5);
        assert_eq!(s[1].time, 1752940801.25);
        assert!(!s[0].cloud && s[1].cloud);
        assert_eq!(s[0].peak_ka, -23.4);
    }
}
