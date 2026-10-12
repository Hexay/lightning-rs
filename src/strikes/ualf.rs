use super::Strike;
use crate::geo::{LatLon, days_from_civil};
use anyhow::{Context, Result, bail};

/// Parses one UALF v0 record (25 whitespace-separated fields; field 8 is nanoseconds).
pub fn parse_line(line: &str) -> Result<Strike> {
    let f: Vec<&str> = line.split_whitespace().collect();
    if f.len() != 25 {
        bail!("UALF line has {} fields, expected 25", f.len());
    }
    let n = |i: usize| -> Result<f64> {
        f[i].parse::<f64>().with_context(|| format!("field {} = {:?}", i + 1, f[i]))
    };
    let days = days_from_civil(n(1)? as i64, n(2)? as u32, n(3)? as u32);
    let time = days as f64 * 86400.0 + n(4)? * 3600.0 + n(5)? * 60.0 + n(6)? + n(7)? * 1e-9;
    Ok(Strike {
        time,
        pos: LatLon::new(n(8)?, n(9)?),
        peak_ka: n(10)? as f32,
        ellipse_angle_deg: n(14)? as f32,
        semi_major_km: n(15)? as f32,
        semi_minor_km: n(16)? as f32,
        cloud: f[21] == "1",
    })
}

pub fn parse(text: &str) -> Result<Vec<Strike>> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| parse_line(l).with_context(|| format!("UALF line {}", i + 1)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frost_record() {
        let s = parse_line(
            "0 2025 07 19 00 00 10 667718400 62.8343 8.6812 50 0 15 19 146.72 0.17 0.10 0.93 23.4 13.7 7.8 0 1 0 1",
        )
        .unwrap();
        assert_eq!(s.pos, LatLon::new(62.8343, 8.6812));
        assert_eq!(s.peak_ka, 50.0);
        assert!(!s.cloud);
        assert!((s.semi_major_km - 0.17).abs() < 1e-6);
        let expected = days_from_civil(2025, 7, 19) as f64 * 86400.0 + 10.6677184;
        assert!((s.time - expected).abs() < 1e-6);
    }
}
