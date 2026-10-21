use crate::model::params::{Atmosphere, ModelKind, Physics};
use crate::search::pipeline::SearchConfig;
use crate::search::feature::Feature;
use crate::search::free_amp::FreeAmp;
use crate::search::scorer::Stage;
use std::path::Path;

/// Declares the TOML-overridable `SearchConfig` fields once; each is optional in the file.
macro_rules! overrides {
    ($($field:ident: $ty:ty),* $(,)?) => {
        #[derive(serde::Deserialize, Default)]
        #[serde(deny_unknown_fields)]
        struct Overrides { $($field: Option<$ty>),* }

        impl Overrides {
            fn apply(self, cfg: &mut SearchConfig) {
                $(if let Some(v) = self.$field { cfg.$field = v; })*
            }
        }
    };
}

overrides! {
    phys: Physics,
    n_eff_factor: f64,
    feature: Feature,
    coarse: Stage,
    mid: Stage,
    fine: Stage,
    coarse_peaks: usize,
    fine_regions: usize,
    mid_half_km: f64,
    fine_half_km: f64,
    robust_lags: usize,
    nuisance: Vec<Atmosphere>,
    min_strikes: usize,
    free_amp: Option<FreeAmp>,
    use_stereo: bool,
}

pub fn search_config(kind: ModelKind, overrides: Option<&Path>) -> anyhow::Result<SearchConfig> {
    let mut cfg = defaults(kind);
    let mut onsets = DEFAULT_CG_ONSET_ALTS_KM.to_vec();
    if let Some(p) = overrides {
        let mut table: toml::Table = toml::from_str(&std::fs::read_to_string(p)?)?;
        let mut take = |key: &str| -> anyhow::Result<Option<Vec<f64>>> { Ok(table.remove(key).map(|v| v.try_into()).transpose()?) };
        let (lapses, onset_override) = (take("lapse_k_per_km")?, take("cg_onset_alt_km")?);
        table.try_into::<Overrides>()?.apply(&mut cfg);
        if let Some(l) = lapses {
            cfg.nuisance = cross(&cfg.nuisance, &l, Atmosphere::with_lapse);
        }
        onsets = onset_override.unwrap_or(onsets);
    }
    cfg.nuisance = cross(&cfg.nuisance, &onsets, Atmosphere::with_cg_onset_alt);
    Ok(cfg)
}

/// Fitted CG onset heights: real thunder times like a source ~6 km up, sims pick 0.
/// See research/empirical_thunder_response.md. `cg_onset_alt_km = [..]` overrides; `[0.0]` disables.
const DEFAULT_CG_ONSET_ALTS_KM: [f64; 3] = [0.0, 3.0, 6.0];

/// Top-level `lapse_k_per_km = [..]` / `cg_onset_alt_km = [..]` in an overrides file cross the
/// nuisance atmospheres with these values.
fn cross(atms: &[Atmosphere], values: &[f64], with: fn(Atmosphere, f64) -> Atmosphere) -> Vec<Atmosphere> {
    atms.iter().flat_map(|&a| values.iter().map(move |&v| with(a, v))).collect()
}

fn defaults(kind: ModelKind) -> SearchConfig {
    SearchConfig {
        kind,
        phys: Physics::default(),
        coarse: Stage { spacing_km: 2.5, rate_hz: 1.0, smooth_s: 2.5 },
        mid: Stage { spacing_km: 0.75, rate_hz: 2.5, smooth_s: 1.0 },
        fine: Stage { spacing_km: 0.5, rate_hz: 10.0, smooth_s: 0.5 },
        min_strikes: 3,
        coarse_peaks: 100,
        fine_regions: 3,
        mid_half_km: 2.5,
        fine_half_km: 12.0,
        fine_time_margin_s: 120.0,
        polish_half_km: 1.5,
        polish_spacing_km: 0.1,
        nuisance: nuisance_grid(),
        floor_window_s: 30.0,
        n_eff_factor: 0.5,
        feature: Feature::Envelope,
        // Off: on the sim tier and real clip it lifted false candidates as much as the truth (no gain).
        robust_lags: 0,
        free_amp: None,
        // Off: on a 0.15 m stereo sim inside a storm it added only a few nats at the truth (see doa.rs).
        use_stereo: false,
    }
}

/// Summer-Nordic prior: 5–25 °C, calm or 6 m/s from 8 directions.
fn nuisance_grid() -> Vec<Atmosphere> {
    let mut winds = vec![(0.0, 0.0)];
    for k in 0..8 {
        let a = k as f64 * std::f64::consts::FRAC_PI_4;
        winds.push((6.0 * a.sin(), 6.0 * a.cos()));
    }
    [5.0, 15.0, 25.0]
        .iter()
        .flat_map(|&t| winds.iter().map(move |&(e, n)| Atmosphere::from_temp(t, e, n)))
        .collect()
}
