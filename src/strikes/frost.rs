use super::{Strike, ualf};
use crate::geo::civil_from_days;
use anyhow::{Context, Result};
use base64::Engine;
use std::path::{Path, PathBuf};

const ENDPOINT: &str = "https://frost.met.no/lightning/v0.ualf";

/// Loads strikes for whole UTC days covering [start, end), downloading each day once into `cache_dir`.
pub fn load_range(start: f64, end: f64, cache_dir: &Path) -> Result<Vec<Strike>> {
    let first_day = (start / 86400.0).floor() as i64;
    let last_day = ((end / 86400.0).ceil() as i64).max(first_day + 1);
    let mut all = Vec::new();
    for day in first_day..last_day {
        let text = cached_day(day, cache_dir)?;
        all.extend(ualf::parse(&text)?.into_iter().filter(|s| s.time >= start && s.time < end));
    }
    Ok(all)
}

fn cached_day(day: i64, cache_dir: &Path) -> Result<String> {
    let (y, m, d) = civil_from_days(day);
    let path: PathBuf = cache_dir.join(format!("{y:04}-{m:02}-{d:02}.ualf"));
    if path.exists() {
        return std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()));
    }
    let (ny, nm, nd) = civil_from_days(day + 1);
    let range = format!("{y:04}-{m:02}-{d:02}T00:00:00Z/{ny:04}-{nm:02}-{nd:02}T00:00:00Z");
    eprintln!("downloading Frost lightning {range}");
    let text = download(&range)?;
    std::fs::create_dir_all(cache_dir)?;
    std::fs::write(&path, &text)?;
    Ok(text)
}

fn download(referencetime: &str) -> Result<String> {
    let client_id = std::env::var("FROST_CLIENT_ID").context("FROST_CLIENT_ID not set (see .env)")?;
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:"));
    let resp = ureq::get(ENDPOINT)
        .query("referencetime", referencetime)
        .set("Authorization", &format!("Basic {auth}"))
        .call();
    match resp {
        // into_string caps at 10 MB; busy storm days exceed it.
        Ok(r) => {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut r.into_reader(), &mut text)?;
            Ok(text)
        }
        // Frost answers 404 for a range with no strikes.
        Err(ureq::Error::Status(404, _)) => Ok(String::new()),
        Err(e) => Err(e).context("Frost request failed"),
    }
}
