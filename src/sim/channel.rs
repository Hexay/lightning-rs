use rand::Rng;
use rand_distr::{Distribution, Normal};

/// One radiating channel segment in listener-centred km (east, north, up) with its unit direction.
pub struct Segment {
    pub pos: [f64; 3],
    pub dir: [f64; 3],
    pub len_km: f64,
}

const SEG_KM: f64 = 0.03;
/// RMS tortuosity per segment, radians (field studies give ~16°).
const TORTUOSITY: f64 = 0.28;

/// Tortuous cloud-to-ground channel rising from `ground` (e, n) to a random top height.
pub fn cloud_to_ground(ground: (f64, f64), rng: &mut impl Rng) -> Vec<Segment> {
    let top = rng.gen_range(4.0..7.5);
    let mut heading = [0.0, 0.0, 1.0];
    let mut p = [ground.0, ground.1, 0.0];
    let mut segs = Vec::new();
    while p[2] < top {
        heading = perturb(heading, rng, 0.9);
        segs.push(step(&mut p, heading));
    }
    segs
}

/// Mostly horizontal in-cloud channel piece centred on `center` (e, n). LLS in-cloud records are
/// individual pulses, so each radiates only its local stretch of the flash, not the whole flash.
pub fn in_cloud(center: (f64, f64), rng: &mut impl Rng) -> Vec<Segment> {
    let alt = rng.gen_range(5.0..9.5);
    let len = rng.gen_range(0.5..2.0);
    let az: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
    let mut heading = [az.sin(), az.cos(), 0.0];
    let mut p = [center.0 - heading[0] * len / 2.0, center.1 - heading[1] * len / 2.0, alt];
    let mut segs = Vec::new();
    for _ in 0..(len / SEG_KM) as usize {
        heading = perturb(heading, rng, 0.0);
        segs.push(step(&mut p, heading));
    }
    segs
}

fn step(p: &mut [f64; 3], dir: [f64; 3]) -> Segment {
    let mid = [p[0] + dir[0] * SEG_KM / 2.0, p[1] + dir[1] * SEG_KM / 2.0, p[2] + dir[2] * SEG_KM / 2.0];
    for k in 0..3 {
        p[k] += dir[k] * SEG_KM;
    }
    Segment { pos: mid, dir, len_km: SEG_KM }
}

/// Random kink, then pulled back toward the preferred vertical component `up_bias` (CG climbs, IC stays level).
fn perturb(d: [f64; 3], rng: &mut impl Rng, up_bias: f64) -> [f64; 3] {
    let n = Normal::new(0.0, TORTUOSITY).unwrap();
    let mut v = [d[0] + n.sample(rng), d[1] + n.sample(rng), d[2] + n.sample(rng)];
    v[2] = 0.8 * v[2] + 0.2 * up_bias;
    let norm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|x| x / norm)
}
