use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum ModelKind {
    /// Extended-source physics, per-strike uncertainty, Pearson likelihood. See README.md#how-it-works.
    Improved,
    /// Reimplementation of the video's method: point source 2 km up, 343 m/s, 1/r² capped, cosine.
    Baseline,
}

/// Speed of sound and horizontal wind; the nuisance parameters fit during refinement.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    pub c_mps: f64,
    pub wind_e_mps: f64,
    pub wind_n_mps: f64,
    /// Temperature lapse rate (K/km) that refracts sound upward; 0 = straight rays, no refraction shadow.
    #[serde(default)]
    pub lapse_k_per_km: f64,
    /// Effective CG onset height under this hypothesis; None uses `Physics::cg_onset_alt_km`.
    #[serde(default)]
    pub cg_onset_alt_km: Option<f64>,
}

impl Atmosphere {
    pub fn from_temp(temp_c: f64, wind_e_mps: f64, wind_n_mps: f64) -> Self {
        Self { c_mps: 331.3 * (1.0 + temp_c / 273.15).sqrt(), wind_e_mps, wind_n_mps, lapse_k_per_km: 0.0, cg_onset_alt_km: None }
    }

    pub fn still(c_mps: f64) -> Self {
        Self { c_mps, wind_e_mps: 0.0, wind_n_mps: 0.0, lapse_k_per_km: 0.0, cg_onset_alt_km: None }
    }

    pub fn with_lapse(self, lapse_k_per_km: f64) -> Self {
        Self { lapse_k_per_km, ..self }
    }

    pub fn with_cg_onset_alt(self, km: f64) -> Self {
        Self { cg_onset_alt_km: Some(km), ..self }
    }

    /// Lowest source height audible at ground range `d_km`: the grazing ray is a circle of radius
    /// 2T/Γ, so it reaches h = d²Γ/(4T). See research/thunder_physics_lit.md.
    pub fn shadow_alt_km(&self, d_km: f64) -> f64 {
        let temp_k = 273.15 * (self.c_mps / 331.3).powi(2);
        d_km * d_km * self.lapse_k_per_km / (4.0 * temp_k)
    }
}

/// Priors for the improved model. Rationale: research/thunder_physics_lit.md.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Physics {
    pub cg_top_km: f64,
    pub cg_amp_alt_km: f64,
    /// Effective height the CG envelope onset is timed from (0 = channel base).
    pub cg_onset_alt_km: f64,
    pub ic_alt_km: f64,
    pub ic_half_len_km: f64,
    pub ic_gain: f64,
    pub min_range_km: f64,
    pub audible_km: f64,
    pub current_exp: f64,
    pub min_decay_s: f64,
    /// Rumble decay constant as a fraction of the channel's arrival-time spread.
    pub decay_frac: f64,
    /// Strikes add incoherently: sum power, then take the root (vs summing amplitudes).
    pub power_sum: bool,
    /// Envelope compression exponent applied to model and observation alike.
    pub gamma: f64,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            cg_top_km: 5.0,
            cg_amp_alt_km: 1.5,
            cg_onset_alt_km: 0.0,
            ic_alt_km: 7.0,
            ic_half_len_km: 1.0,
            ic_gain: 0.5,
            min_range_km: 1.0,
            audible_km: 20.0,
            current_exp: 0.5,
            min_decay_s: 1.0,
            decay_frac: 0.5,
            power_sum: true,
            gamma: 0.5,
        }
    }
}

impl Physics {
    /// Exponent applied to the rendered sum so it is comparable with RMS^gamma of the audio.
    pub fn render_exponent(&self) -> f64 {
        if self.power_sum { self.gamma / 2.0 } else { self.gamma }
    }
}

impl Physics {
    /// Strikes beyond this contribute < ~1% of an audible strike; skip them.
    pub fn query_radius_km(&self) -> f64 {
        self.audible_km * 1.8
    }
}

pub const BASELINE_SOURCE_ALT_KM: f64 = 2.0;
pub const BASELINE_C_MPS: f64 = 343.0;
pub const BASELINE_CAP_RANGE_KM: f64 = 1.0;
pub const BASELINE_RADIUS_KM: f64 = 40.0;
