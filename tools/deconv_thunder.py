"""Empirical thunder response at a known spot: regress the observed envelope (power domain) on
per-distance-bin kernels placed at each strike's straight-line arrival d/c, weighted by |kA|/d².
Usage: py -3 -I tools/deconv_thunder.py OUT.png LAT LON UALF label=probe.csv@START [label=...]
probe.csv comes from `probe --start START` (t, observed, model at 10 Hz; observed is envelope^0.5)."""
import calendar
import csv
import math
import sys

import matplotlib
import numpy as np

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

C_KMS = 0.340
PRE_S, POST_S, TAP_S = 6.0, 20.0, 0.5
BINS = [("CG 4–9", False, 4, 9), ("CG 9–13", False, 9, 13), ("CG 13–20", False, 13, 20), ("IC <20", True, 0, 20)]
RIDGE = 1e-3


def strikes(path, lat, lon):
    kx = 111.32 * math.cos(math.radians(lat))
    out = []
    for line in open(path):
        p = line.split()
        if len(p) < 22:
            continue
        d = math.hypot((float(p[8]) - lat) * 111.32, (float(p[9]) - lon) * kx)
        if d < 25:
            t = calendar.timegm(tuple(int(x) for x in p[1:7])) + int(p[7]) * 1e-9
            out.append((t + d / C_KMS, d, p[21] == "1", abs(float(p[10]))))
    return out


def load_probe(probe):
    with open(probe) as f:
        rows = list(csv.DictReader(f))
    rate = 1 / (float(rows[1]["t"]) - float(rows[0]["t"]))
    return np.array([float(r["observed"]) for r in rows]) ** 2, rate  # undo gamma 0.5 → amplitude envelope


def fit(probe, start, sts, bins=BINS, cg_alt_km=0.0):
    y, rate = load_probe(probe)
    n_t, n_tap = len(y), int((PRE_S + POST_S) / TAP_S)
    X = np.zeros((n_t, len(bins) * n_tap + 1))
    X[:, -1] = 1.0
    counts = [0] * len(bins)
    for arr, d, cloud, ka in sts:
        b = next((k for k, (_, c, lo, hi) in enumerate(bins) if c == cloud and lo <= d < hi), None)
        if b is None:
            continue
        rel = arr - start + (0.0 if cloud else (math.hypot(d, cg_alt_km) - d) / C_KMS)
        if rel < -POST_S or rel > n_t / rate + PRE_S:
            continue
        counts[b] += 1
        w = max(ka, 1.0) / max(d, 1.0) ** 2
        for j in range(n_tap):
            lo = int((rel - PRE_S + j * TAP_S) * rate)
            lo, hi = max(lo, 0), min(lo + int(TAP_S * rate), n_t)
            if lo < hi:
                X[lo:hi, b * n_tap + j] += w
    scale = np.linalg.norm(X, axis=0) + 1e-12
    Xs = X / scale
    coef = np.linalg.solve(Xs.T @ Xs + RIDGE * np.eye(Xs.shape[1]), Xs.T @ y) / scale
    resid = y - X @ coef
    r2 = 1 - resid.var() / y.var()
    kern = coef[:-1].reshape(len(bins), n_tap)
    return kern, counts, r2


SHARED = [("CG", False, 0, 25), ("IC", True, 0, 25)]


def scan_alt(probe, start, sts):
    """R² with one shared CG kernel when CG arrivals are timed from height h; peak = effective onset height."""
    return [(h / 2, fit(probe, start, sts, SHARED, h / 2)[2]) for h in range(0, 17)]


def main():
    out, lat, lon, ualf = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4]
    sts = strikes(ualf, lat, lon)
    t = np.arange(int((PRE_S + POST_S) / TAP_S)) * TAP_S - PRE_S + TAP_S / 2
    fig, axes = plt.subplots(1, len(BINS), figsize=(4 * len(BINS), 3.2))
    for k, arg in enumerate(sys.argv[5:]):
        label, rest = arg.split("=", 1)
        probe, start = rest.rsplit("@", 1)
        kern, counts, r2 = fit(probe, float(start), sts)
        print(f"{label}: R² {r2:.2f}")
        scan = scan_alt(probe, float(start), sts)
        print("  shared-kernel R² by CG onset height: " + " ".join(f"{h:g}:{r:.3f}" for h, r in scan))
        for b, ax in enumerate(axes):
            kb = kern[b] / (np.abs(kern[b]).max() + 1e-12)
            ax.plot(t, kb, color=f"C{k}", label=f"{label} (n={counts[b]})")
            cum = np.cumsum(np.clip(kern[b], 0, None))
            half = t[np.searchsorted(cum, 0.5 * cum[-1])] if cum[-1] > 0 else float("nan")
            print(f"  {BINS[b][0]:9s} n {counts[b]:3d}: peak {t[np.argmax(kern[b])]:+5.1f} s, energy midpoint {half:+5.1f} s after d/c")
    for b, ax in enumerate(axes):
        ax.axvline(0, color="k", lw=0.5)
        ax.set_title(BINS[b][0] + " km")
        ax.set_xlabel("s after d/c")
        ax.legend(fontsize=7)
    fig.tight_layout()
    fig.savefig(out, dpi=110)


if __name__ == "__main__":
    main()
