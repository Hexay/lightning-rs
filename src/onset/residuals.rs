use super::detect::{Onset, detect};
use crate::audio::BandAudio;
use crate::geo::LatLon;
use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::model::propagation::arrival;
use crate::strikes::StrikeIndex;

const MATCH_S: f64 = 4.0;
const DIST_BINS_KM: [f64; 6] = [0.0, 5.0, 10.0, 15.0, 20.0, 40.0];

struct Pair {
    onset: Onset,
    dist_km: f64,
    resid_s: f64,
    cloud: bool,
    peak_ka: f32,
    pos: LatLon,
}

/// Pairs each detected onset with the loudest strike predicted within ±MATCH_S at `p` (clip starts at
/// `start`), writes out/residuals.csv and prints mean residual (observed − predicted) by distance.
pub fn report(index: &StrikeIndex, phys: &Physics, atm: &Atmosphere, audio: &BandAudio, p: LatLon, start: f64) {
    let mut ids = Vec::new();
    index.within(&p, phys.query_radius_km(), &mut ids);
    let arrivals: Vec<_> = ids
        .iter()
        .map(|&i| &index.strikes[i as usize])
        .map(|s| (s, arrival(ModelKind::Improved, phys, atm, s, &p)))
        .filter(|(_, a)| a.audible && a.onset >= start - MATCH_S && a.onset <= start + audio.duration_s + MATCH_S)
        .collect();
    let pairs: Vec<Pair> = detect(audio, 48)
        .into_iter()
        .filter_map(|o| {
            let (s, a) = arrivals
                .iter()
                .filter(|(_, a)| (a.onset - start - o.t).abs() <= MATCH_S)
                .max_by(|x, y| x.1.area.total_cmp(&y.1.area))?;
            Some(Pair { onset: o, dist_km: p.dist_km(&s.pos), resid_s: start + o.t - a.onset, cloud: s.cloud, peak_ka: s.peak_ka, pos: s.pos })
        })
        .collect();

    let rows: String = pairs
        .iter()
        .map(|q| {
            format!(
                "{:.2},{:.3},{:.2},{:.3},{},{:.1},{:.5},{:.5}\n",
                q.onset.t, q.onset.strength, q.dist_km, q.resid_s, q.cloud as u8, q.peak_ka, q.pos.lat, q.pos.lon
            )
        })
        .collect();
    let _ = std::fs::create_dir_all("out");
    let _ = std::fs::write("out/residuals.csv", format!("t,strength,dist_km,resid_s,cloud,peak_ka,lat,lon\n{rows}"));

    println!("residuals: {} of {} audible strikes matched to onsets (obs − pred, s)", pairs.len(), arrivals.len());
    for w in DIST_BINS_KM.windows(2) {
        let r: Vec<f64> = pairs.iter().filter(|q| q.dist_km >= w[0] && q.dist_km < w[1]).map(|q| q.resid_s).collect();
        if r.is_empty() {
            continue;
        }
        let mean = r.iter().sum::<f64>() / r.len() as f64;
        let mut sorted = r.clone();
        sorted.sort_by(f64::total_cmp);
        println!("  {:>4.0}–{:<4.0} km: n {:>2}  mean {:+.2}  median {:+.2}", w[0], w[1], r.len(), mean, sorted[r.len() / 2]);
    }
}
