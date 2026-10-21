"""Proposes sim-suite cases where the listener hears enough strikes to be locatable.

Candidate listeners sit 5-15 km from active strikes; each (listener, start) is scored by the number of
strikes within 20 km whose sound arrives inside a 260 s clip and by how widely their bearings spread.
Usage: py -3 tools/pick_sim_cases.py data/frost/2025-07-19.ualf [n_cases]
"""
import sys

import numpy as np

CLIP_S, AUDIBLE_KM, C_KMS = 260.0, 20.0, 0.340
ualf = sys.argv[1]
n_cases = int(sys.argv[2]) if len(sys.argv) > 2 else 8
rows = []
for line in open(ualf):
    p = line.split()
    t = int(p[4]) * 3600 + int(p[5]) * 60 + int(p[6]) + int(p[7]) * 1e-9
    rows.append((t, float(p[8]), float(p[9])))
s = np.array(rows)
t, lat, lon = s[:, 0], s[:, 1], s[:, 2]
rng = np.random.default_rng(0)


def heard(la, lo, start):
    k = np.cos(np.radians(la)) * 111.2
    e, n = (lon - lo) * k, (lat - la) * 111.2
    d = np.hypot(e, n)
    m = (d <= AUDIBLE_KM) & (t + d / C_KMS >= start) & (t + d / C_KMS <= start + CLIP_S)
    if m.sum() == 0:
        return 0, 0.0
    az = np.arctan2(e[m], n[m])
    r = np.hypot(np.cos(az).mean(), np.sin(az).mean())
    return int(m.sum()), float(np.degrees(np.sqrt(-2 * np.log(max(r, 1e-9)))))


cands = []
for i in rng.choice(len(t), size=min(4000, len(t)), replace=False):
    ang, dist = rng.uniform(0, 2 * np.pi), rng.uniform(5, 15)
    la = lat[i] + dist * np.cos(ang) / 111.2
    lo = lon[i] + dist * np.sin(ang) / (111.2 * np.cos(np.radians(lat[i])))
    start = np.floor((t[i] - 30) / 10) * 10
    n, spread = heard(la, lo, start)
    if n >= 30:
        cands.append((n, spread, la, lo, start))

# Diverse picks: alternate one-sided (spread < 40°) and surrounded, ≥ 50 km or ≥ 1 h apart.
cands.sort(reverse=True)
picked = []
for want_wide in [False, True] * n_cases:
    for c in cands:
        if (c[1] >= 40) != want_wide or any(
            np.hypot((c[2] - p[2]) * 111.2, (c[3] - p[3]) * 55) < 50 and abs(c[4] - p[4]) < 3600 for p in picked
        ):
            continue
        picked.append(c)
        break
    if len(picked) == n_cases:
        break
for i, (n, spread, la, lo, start) in enumerate(picked):
    hms = f"{int(start // 3600):02d}:{int(start % 3600 // 60):02d}:{int(start % 60):02d}"
    print(f'  "case{i + 1}_{"wide" if spread >= 40 else "side"} {la:.3f} {lo:.3f} {hms}"  # heard {n}, spread {spread:.0f} deg')
