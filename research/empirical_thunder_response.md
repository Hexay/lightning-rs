# Empirical thunder response (vid2, 2026-10-13/14)

Method: `tools/deconv_thunder.py`. It regresses the observed envelope (10 Hz probe output at a known spot and start) on per-distance-bin FIR kernels placed at each catalog strike's straight-line arrival d/c, weighted by |kA|/d². Overlapping strikes are handled jointly. Inputs come from `lightning-rs probe --start EPOCH` → out/probe.csv.

## Per-bin kernels (vid2 at truth; start 15:59:48.6 fitted at h=0)

| CG range | real peak after d/c | twin peak |
|---|---|---|
| 4–9 km (n 15) | +2.8 s | +0.2 s |
| 9–13 km (n 49) | +1.2 s | +0.2 s |

A constant clock or position error would shift every bin equally. The near-minus-far difference is what matters: ≈1.6 s real, 0 twin.

## Shared CG kernel, arrival timed from height h

R² as a function of h:

| h (km) | 0 | 2 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|
| real | 0.461 | 0.475 | 0.533 | 0.570 | **0.602** | 0.543 | 0.248 |
| twin (h=0 physics) | **0.778** | 0.777 | 0.759 | 0.728 | 0.636 | 0.418 | 0.163 |

The method recovers h≈0 on the twin. Real thunder behaves like a compact source ≈6 km up.

Sim variants with a quiet lower channel (`sim --cg-quiet-below 3|5`) give a broad, late hump for near strikes and still peak at h≈0–3 km. They don't reproduce the real response.

## Effect on location (vid2, not blind: truth revealed 2026-10-13)

| fixed CG onset height | error | cr90 |
|---|---|---|
| 0 (default) | 2.64 km | 34 km² |
| 4 | 1.75 km | 24 km² |
| 5 | 1.59 km | 39 km² |
| 6 | 1.34 km | 46 km² |

## Truth-free: h fitted as a search nuisance (`experiments/onset_grid_ms20.toml`, run on testbox)

Grid is h ∈ {0, 2, 4, 6, 8} crossed with the 27 atmospheres, on top of min_strikes 20.

| case | min_strikes_20_grid | + onset grid | chosen h |
|---|---|---|---|
| real vid2 | 2.64 km | **1.77 km** | 4 |
| real_twin | 1.43 | 1.43 | 0 |
| loc_wide_62N8E | 0.48 | 0.78 | 2 |
| loc_wide_62N9E | 0.96 | 0.82 | 0 |
| loc_wide_65N19E | 0.41 | 0.41 | 0 |
| loc_wide_62N10E | 476 (lost) | 479 (lost) | 4 |

- Credible level of the truth: 0.93 → 0.66.
- Runtime: 191 → 643 s.

### Adopted default (2026-10-20): h ∈ {0, 3, 6}

`config.rs` DEFAULT_CG_ONSET_ALTS_KM; an override `cg_onset_alt_km = [0.0]` restores the old behaviour.

| case | min_strikes 20 + 3-grid | default (min_strikes 3) + 3-grid |
|---|---|---|
| real vid2 | 1.33 km | 1.33 km (cr90 44.5 km², truth credible level 0.59) |
| real_twin | 1.43 | 1.43 |
| loc_wide 62N8E / 62N9E / 65N19E | 0.48 / 0.96 / 0.41 (= baseline, h=0) | — |

## Ruled out as the cause

- Refraction shadow D²Γ/(4T): real 2.55–2.89 km.
- Terrain shielding: the strikes are over Røssvatnet and the line of sight is clear.
- Clock-rate drift: corr(t, distance) = +0.10.

See also thunder_physics_lit.md: infrasound source heights have a median of 3.8–4.5 km (Rusz 2021).

## Caveats

- Only one clip with an exact truth.
- Other YouTube clips (real_clip_candidates.md) heard too few strikes or look edited.
- h is fitted at the truth, so the truth-free route is fitting h as a search nuisance (`experiments/onset_grid.toml`).
