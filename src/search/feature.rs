use serde::Deserialize;

/// Which transform of the envelope is correlated. Applied identically to model and observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feature {
    Envelope,
    /// Half-wave rectified rise over one smoothing width: keys on onsets, discounts decay shape.
    Rise,
}

impl Feature {
    pub fn apply(self, x: &mut Vec<f32>, rate: f64, smooth_s: f64) {
        if self == Feature::Envelope {
            return;
        }
        let d = ((smooth_s.max(0.2) * rate).round() as usize).max(1);
        for i in (0..x.len()).rev() {
            x[i] = if i >= d { (x[i] - x[i - d]).max(0.0) } else { 0.0 };
        }
    }
}
