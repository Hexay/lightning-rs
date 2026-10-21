"""Fit resid = t0 + (hypot(d, h) - d) / c to a probe residuals CSV (robust L1 over a grid of h).
Usage: py -3 -I tools/fit_onset_height.py out/residuals-real.csv [more.csv ...]"""
import csv
import math
import random
import sys

C_KMS = 0.340


def cost_at(rows, h):
    lag = [r["resid_s"] - (math.hypot(r["dist_km"], h) - r["dist_km"]) / C_KMS for r in rows]
    t0 = sorted(lag)[len(lag) // 2]
    return sum(abs(x - t0) for x in lag), t0


def fit(rows):
    return min((cost_at(rows, h10 / 10)[0], h10 / 10, cost_at(rows, h10 / 10)[1]) for h10 in range(0, 81))


def bootstrap_h(rows, n=300):
    rng = random.Random(0)
    hs = sorted(fit([rng.choice(rows) for _ in rows])[1] for _ in range(n))
    return hs[int(0.05 * n)], hs[int(0.95 * n)]


for path in sys.argv[1:]:
    with open(path) as f:
        rows = [{k: float(v) for k, v in r.items()} for r in csv.DictReader(f)]
    rows = [r for r in rows if abs(r["resid_s"]) < 3.5]
    cg = [r for r in rows if r["cloud"] == 0]
    for name, sub in (("all", rows), ("cg", cg)):
        if len(sub) < 5:
            continue
        cost, h, t0 = fit(sub)
        lo, hi = bootstrap_h(sub)
        n = len(sub)
        print(f"{path} [{name}] n {n}: h {h:.1f} km (90% {lo:.1f}–{hi:.1f})  t0 {t0:+.2f} s  "
              f"L1/n {cost / n:.3f} vs h=0 {cost_at(sub, 0.0)[0] / n:.3f}")
