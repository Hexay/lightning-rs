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
    if let Some(p) = overrides {
        toml::from_str::<Overrides>(&std::fs::read_to_string(p)?)?.apply(&mut cfg);
    }
    Ok(cfg)
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
