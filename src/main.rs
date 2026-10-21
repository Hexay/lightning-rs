use anyhow::Result;
use clap::{Parser, Subcommand};
use lightning_rs::model::params::ModelKind;
use lightning_rs::{audio, config, eval, geo, model, onset, output, search, sim, strikes};
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(about = "Locate a recording from the thunder in it, using lightning detection data")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download (and cache) Frost lightning for a UTC day.
    Fetch {
        #[arg(long)]
        date: String,
    },
    /// Search all strikes on `date` for where and when `audio` was recorded.
    Locate {
        #[arg(long)]
        audio: PathBuf,
        /// UTC date the recording was made (YYYY-MM-DD); time of day is searched.
        #[arg(long)]
        date: String,
        /// Known recording start of the audio, HH:MM:SS[.sss] UTC; searched ±0.5 s.
        /// Gotcha: video flash times give the frame clock, which can be seconds off the audio track.
        #[arg(long)]
        start: Option<String>,
        #[command(flatten)]
        model: ModelArgs,
        #[arg(long, default_value = "out/run")]
        out: PathBuf,
    },
    /// Synthesise a thunder recording from real strikes at a chosen listener (writes audio.wav + truth.toml).
    Sim {
        #[arg(long)]
        date: String,
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
        /// Recording start, HH:MM:SS UTC.
        #[arg(long)]
        start: String,
        #[arg(long, default_value_t = 260.0)]
        duration: f64,
        #[arg(long, default_value_t = 14.0)]
        temp: f64,
        #[arg(long, default_value_t = 0.0)]
        wind_e: f64,
        #[arg(long, default_value_t = 0.0)]
        wind_n: f64,
        /// Temperature lapse rate, K/km (0 = no refraction shadow).
        #[arg(long, default_value_t = 0.0)]
        lapse: f64,
        /// CG channel radiates weakly below this height, km (0 = uniform).
        #[arg(long, default_value_t = 0.0)]
        cg_quiet_below: f64,
        #[arg(long, default_value_t = 0.02)]
        noise: f64,
        #[arg(long, default_value_t = 4)]
        cars: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Two-mic spacing in metres (0 = mono output).
        #[arg(long, default_value_t = 0.0)]
        mic_spacing: f64,
        /// Compass bearing of the mic axis (right → left mic).
        #[arg(long, default_value_t = 0.0)]
        mic_axis: f64,
        #[arg(long)]
        out: PathBuf,
    },
    /// Diagnostic: score one point at each stage, over the whole day and (optionally) at a fixed start.
    Probe {
        #[arg(long)]
        audio: PathBuf,
        #[arg(long)]
        date: String,
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
        /// Known start (epoch seconds), e.g. from a sim truth file.
        #[arg(long)]
        start: Option<f64>,
        /// Time each scoring phase instead of scoring.
        #[arg(long)]
        bench: bool,
        #[command(flatten)]
        model: ModelArgs,
    },
    /// Compare a finished run against a held-out truth file.
    Eval {
        #[arg(long)]
        run: PathBuf,
        #[arg(long, default_value = "eval/truth.toml")]
        truth: PathBuf,
    },
}

#[derive(clap::Args)]
struct ModelArgs {
    #[arg(long, value_enum, default_value = "improved")]
    model: ModelKind,
    /// TOML overrides of search settings (see config.rs `overrides!`).
    #[arg(long)]
    overrides: Option<PathBuf>,
}

impl ModelArgs {
    fn config(&self) -> Result<search::pipeline::SearchConfig> {
        config::search_config(self.model, self.overrides.as_deref())
    }
}

const CACHE_DIR: &str = "data/frost";
/// Search half-width around a supplied --start (video frame timing is ~0.03 s; this covers clock slop).
const KNOWN_START_TOL_S: f64 = 0.5;

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    match Cli::parse().cmd {
        Cmd::Fetch { date } => {
            let day = geo::parse_date(&date)?;
            let n = strikes::frost::load_range(day, day + 86400.0, CACHE_DIR.as_ref())?.len();
            println!("{n} strikes on {date}");
        }
        Cmd::Locate { audio, date, start, model, out } => locate(&audio, &date, start.as_deref(), &model, &out)?,
        Cmd::Sim { date, lat, lon, start, duration, temp, wind_e, wind_n, lapse, cg_quiet_below, noise, cars, seed, mic_spacing, mic_axis, out } => {
            let day = geo::parse_date(&date)?;
            let sc = sim::Scenario {
                listener: geo::LatLon::new(lat, lon),
                start: geo::parse_hms(day, &start)?,
                duration_s: duration,
                atm: model::params::Atmosphere::from_temp(temp, wind_e, wind_n).with_lapse(lapse),
                noise_level: noise,
                car_passes: cars,
                seed,
                mics: (mic_spacing > 0.0).then_some(sim::Mics { spacing_m: mic_spacing, axis_deg: mic_axis }),
                cg_quiet_below_km: cg_quiet_below,
            };
            let strikes = strikes::frost::load_range(day, day + 86400.0, CACHE_DIR.as_ref())?;
            std::fs::create_dir_all(&out)?;
            sim::write_wav(&out.join("audio.wav"), &sim::synthesize(&strikes, &sc))?;
            let truth = format!(
                "lat = {lat}\nlon = {lon}\nstart = {}\n# temp_c = {temp}, wind_e = {wind_e}, wind_n = {wind_n}, lapse = {lapse}, cg_quiet_below = {cg_quiet_below}, seed = {seed}\n",
                sc.start
            );
            std::fs::write(out.join("truth.toml"), truth)?;
            println!("wrote {}", out.display());
        }
        Cmd::Probe { audio, date, lat, lon, start, bench, model } => {
            let day = geo::parse_date(&date)?;
            let index = strikes::StrikeIndex::new(strikes::frost::load_range(day, day + 86400.0, CACHE_DIR.as_ref())?);
            let audio = audio::load_band(&audio, 20.0, 250.0)?;
            let (cfg, p, day) = (model.config()?, geo::LatLon::new(lat, lon), (day, day + 86400.0));
            if bench {
                search::probe::bench(&cfg, &index, &audio, day, p, start);
            } else {
                let best = search::probe::print(&cfg, &index, &audio, day, p, start);
                let atm = model::params::Atmosphere::from_temp(15.0, 0.0, 0.0);
                onset::residuals::report(&index, &cfg.phys, &atm, &audio, p, start.unwrap_or(best));
            }
        }
        Cmd::Eval { run, truth } => eval::report(&run, &eval::load_truth(&truth)?)?,
    }
    Ok(())
}

fn locate(audio_path: &std::path::Path, date: &str, start: Option<&str>, args: &ModelArgs, out: &std::path::Path) -> Result<()> {
    let model = args.model;
    let t0 = Instant::now();
    let day = geo::parse_date(date)?;
    let index = strikes::StrikeIndex::new(strikes::frost::load_range(day, day + 86400.0, CACHE_DIR.as_ref())?);
    let audio = audio::load_band(audio_path, 20.0, 250.0)?;
    eprintln!(
        "{} strikes, {:.1} s audio ({}), model {model:?} (load {:.1}s)",
        index.strikes.len(),
        audio.duration_s,
        audio.lags.as_ref().map_or("mono".into(), |t| format!("stereo, {} usable lag windows", t.samples.len())),
        t0.elapsed().as_secs_f64()
    );
    let cfg = args.config()?;
    let window = match start {
        Some(s) => {
            let s = geo::parse_hms(day, s)?;
            (s - KNOWN_START_TOL_S, s + KNOWN_START_TOL_S + audio.duration_s)
        }
        None => (day, day + 86400.0),
    };
    let result = search::pipeline::run(&cfg, &index, &audio, window);
    let run_info = json!({
        "model": format!("{model:?}").to_lowercase(),
        "audio": audio_path.file_name().map(|f| f.to_string_lossy().to_string()),
        "date": date,
        "known_start": start,
        "elapsed_s": t0.elapsed().as_secs_f64(),
        "stages": [cfg.coarse, cfg.mid, cfg.fine],
        "physics": cfg.phys,
    });
    let onset_solutions = if model == ModelKind::Improved { solve_onsets(&cfg, &index, &audio, &result) } else { vec![] };
    let traces = search::trace::region_traces(&cfg, &index, &audio, &result.regions);
    let spec = audio::spectrogram::load(audio_path)?;
    output::write_all(out, &result, &onset_solutions, &traces, &spec, run_info)?;
    for s in &onset_solutions {
        println!(
            "onset solve: {:.5},{:.5} ±{:.2}/{:.2} km  start {}  {}/{} onsets matched, rms {:.2}s",
            s.pos.lat, s.pos.lon, s.sigma_e_km, s.sigma_n_km, s.start_utc, s.inliers, s.onsets, s.rms_resid_s
        );
    }
    for r in &result.regions {
        println!(
            "#{} {:.5},{:.5}  P {:.1}%  rho {:.3}  start {}  c {:.1}  cr90 {:.2} km²  heard {}{}",
            r.rank + 1,
            r.map.lat,
            r.map.lon,
            r.region_prob * 100.0,
            r.score.best_rho,
            r.start_utc,
            r.atmosphere.c_mps,
            r.cr90_area_km2,
            r.shape.audible_strikes,
            if r.shape.reliable() { "" } else { "  [UNRELIABLE: too few strikes heard]" }
        );
    }
    eprintln!("wrote {} in {:.1}s total", out.display(), t0.elapsed().as_secs_f64());
    Ok(())
}

/// Independent cross-check: onset consensus solve seeded by each envelope region.
fn solve_onsets(
    cfg: &search::pipeline::SearchConfig,
    index: &strikes::StrikeIndex,
    audio: &audio::BandAudio,
    result: &search::pipeline::SearchResult,
) -> Vec<onset::solve::OnsetSolution> {
    let onsets = onset::detect::detect(audio, 48);
    eprintln!("  onsets detected: {}", onsets.len());
    let p = onset::solve::SolveParams { half_km: 15.0, spacing_km: 0.25, margin_s: 180.0, tol_s: 1.0, onset_sigma_s: 0.4 };
    result
        .regions
        .iter()
        .filter_map(|r| {
            onset::solve::solve(index, &cfg.phys, &r.atmosphere, &onsets, r.map, r.score.best_start, audio.duration_s, &p)
        })
        .collect()
}
