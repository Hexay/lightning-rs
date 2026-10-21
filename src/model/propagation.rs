use super::params::*;
use crate::geo::LatLon;
use crate::strikes::Strike;

/// What one strike contributes at a listener: an onset-shaped pulse of given area.
#[derive(Clone, Copy, Debug)]
pub struct Arrival {
    /// Epoch seconds the first sound reaches the listener.
    pub onset: f64,
    /// Exponential rumble decay; 0 means a plain Gaussian pulse (baseline).
    pub decay_s: f64,
    /// Timing uncertainty from the strike's position error.
    pub sigma_s: f64,
    /// Integrated envelope contribution.
    pub area: f64,
    /// Within audible range, so it counts toward the strikes-per-window minimum.
    pub audible: bool,
    /// Compass bearing from listener to strike.
    pub bearing_rad: f64,
}

pub fn arrival(kind: ModelKind, phys: &Physics, atm: &Atmosphere, strike: &Strike, listener: &LatLon) -> Arrival {
    match kind {
        ModelKind::Improved => improved(phys, atm, strike, listener),
        ModelKind::Baseline => baseline(strike, listener),
    }
}

fn improved(phys: &Physics, atm: &Atmosphere, s: &Strike, listener: &LatLon) -> Arrival {
    let (e, n) = s.pos.enu_km(listener);
    let d = e.hypot(n);
    let (ue, un) = if d > 1e-6 { (e / d, n / d) } else { (1.0, 0.0) };
    let c_kms = (atm.c_mps + atm.wind_e_mps * ue + atm.wind_n_mps * un) / 1000.0;

    let refract = atm.lapse_k_per_km > 0.0;
    let h_min = atm.shadow_alt_km(d);
    let (r_on, r_end, r_amp, gain, lit_frac) = if s.cloud {
        let near = (d - phys.ic_half_len_km).max(0.0);
        let lit = if h_min < phys.ic_alt_km { 1.0 } else { 0.0 };
        (near.hypot(phys.ic_alt_km), (d + phys.ic_half_len_km).hypot(phys.ic_alt_km), d.hypot(phys.ic_alt_km), phys.ic_gain, lit)
    } else {
        let lit = ((phys.cg_top_km - h_min) / phys.cg_top_km).clamp(0.0, 1.0);
        let on_alt = atm.cg_onset_alt_km.unwrap_or(phys.cg_onset_alt_km).max(h_min.min(phys.cg_top_km));
        let amp_alt = phys.cg_amp_alt_km.max(h_min.min(phys.cg_top_km));
        (d.hypot(on_alt), d.hypot(phys.cg_top_km), d.hypot(amp_alt), 1.0, lit)
    };
    let onset = s.time + r_on / c_kms;
    let decay_s = ((r_end - r_on) / c_kms * phys.decay_frac).max(phys.min_decay_s);
    let shadow = if refract { lit_frac } else { 1.0 / (1.0 + (d / phys.audible_km).powi(4)) };
    let current = (s.peak_ka.abs() as f64).max(1.0).powf(phys.current_exp);
    let amp = gain * current * shadow / r_amp.max(phys.min_range_km);
    Arrival {
        onset,
        decay_s,
        sigma_s: s.sigma_along_km(ue, un) / c_kms,
        area: if phys.power_sum { amp * amp } else { amp } * decay_s,
        audible: d <= phys.audible_km && lit_frac > 0.0,
        bearing_rad: (-e).atan2(-n),
    }
}

fn baseline(s: &Strike, listener: &LatLon) -> Arrival {
    let (e, n) = listener.enu_km(&s.pos);
    let r = e.hypot(n).hypot(BASELINE_SOURCE_ALT_KM);
    Arrival {
        onset: s.time + r * 1000.0 / BASELINE_C_MPS,
        decay_s: 0.0,
        sigma_s: 0.0,
        area: 1.0 / r.max(BASELINE_CAP_RANGE_KM).powi(2),
        audible: r <= BASELINE_RADIUS_KM,
        bearing_rad: e.atan2(n),
    }
}
