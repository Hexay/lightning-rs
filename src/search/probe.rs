use super::pipeline::{SearchConfig, observed_feature, scorer};
use super::scorer::Workspace;
use crate::audio::BandAudio;
use crate::geo::{LatLon, format_utc};
use crate::model::params::{Atmosphere, BASELINE_C_MPS, ModelKind};
use crate::strikes::StrikeIndex;

/// Prints each stage's whole-day score at `p`, and the score with the start pinned to `start`.
pub fn print(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, day: (f64, f64), p: LatLon, start: Option<f64>) {
    let atm = match cfg.kind {
        ModelKind::Improved => Atmosphere::from_temp(15.0, 0.0, 0.0),
        ModelKind::Baseline => Atmosphere::still(BASELINE_C_MPS),
    };
    let mut ws = Workspace::default();
    for (name, stage) in [("coarse", cfg.coarse), ("mid", cfg.mid), ("fine", cfg.fine)] {
        let day_s = scorer(cfg, index, audio, stage, (day.0, day.1 - audio.duration_s)).score(&p, &atm, &mut ws);
        print!(
            "{name:>6}: day  rho {:.3} log_ev {:7.2} doa {:.1} start {} ({} strikes)",
            day_s.best_rho,
            day_s.log_ev,
            day_s.doa_ll,
            format_utc(day_s.best_start),
            day_s.strikes_in_window
        );
        if let Some(t) = start {
            let pinned = scorer(cfg, index, audio, stage, (t, t + 0.5 / stage.rate_hz)).score(&p, &atm, &mut ws);
            print!(" | pinned rho {:.3} doa {:.1} ({} strikes)", pinned.best_rho, pinned.doa_ll, pinned.strikes_in_window);
        }
        println!();
    }
    if let Some(t) = start {
        dump_fine(cfg, index, audio, p, &atm, t);
    }
}

/// Single-threaded per-call cost of each scoring phase, per stage, on a 10×10 grid around `p`.
pub fn bench(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, day: (f64, f64), p: LatLon, start: Option<f64>) {
    let atm = Atmosphere::from_temp(15.0, 0.0, 0.0);
    let day_range = (day.0, day.1 - audio.duration_s);
    let mid_day = start.unwrap_or(0.5 * (day.0 + day.1));
    let stages = [
        ("coarse", cfg.coarse, day_range),
        ("mid", cfg.mid, day_range),
        ("fine", cfg.fine, (mid_day - cfg.fine_time_margin_s, mid_day + cfg.fine_time_margin_s)),
    ];
    for (name, stage, range) in stages {
        let sc = scorer(cfg, index, audio, stage, range);
        let cells = super::grid::around(p, 5.0 * stage.spacing_km, stage.spacing_km);
        let mut ws = Workspace::default();
        let t = std::time::Instant::now();
        for c in &cells {
            sc.score(&c.center, &atm, &mut ws);
        }
        let n = cells.len() as f64;
        let ms = |s: f64| s / n * 1e3;
        println!(
            "{name:>6}: {:.2} ms/call  [query+arrivals {:.2}, render {:.2}, correlate {:.2}, reduce {:.2}]",
            ms(t.elapsed().as_secs_f64()),
            ms(ws.phase_s[0]),
            ms(ws.phase_s[1]),
            ms(ws.phase_s[2]),
            ms(ws.phase_s[3])
        );
    }
}

/// Writes out/probe.csv: time, observed, model (fine stage, start pinned), both scaled to unit max.
fn dump_fine(cfg: &SearchConfig, index: &StrikeIndex, audio: &BandAudio, p: LatLon, atm: &Atmosphere, t: f64) {
    let st = cfg.fine;
    let sc = scorer(cfg, index, audio, st, (t, t + 0.5 / st.rate_hz));
    let model = sc.model_window(&p, atm, &mut Workspace::default());
    let obs = observed_feature(cfg, audio, st);
    let norm = |v: &[f32]| v.iter().fold(1e-30f32, |m, x| m.max(*x));
    let (mo, mm) = (norm(&obs), norm(&model));
    let rows: String = (0..obs.len().min(model.len()))
        .map(|i| format!("{:.2},{:.4},{:.4}\n", i as f64 / st.rate_hz, obs[i] / mo, model[i] / mm))
        .collect();
    let _ = std::fs::create_dir_all("out");
    let _ = std::fs::write("out/probe.csv", format!("t,observed,model\n{rows}"));
}
