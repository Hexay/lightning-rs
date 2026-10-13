"""Overlay several probe CSVs (observed curves + the model from the first) into one PNG.
Usage: plot_compare.py out.png label1=a.csv label2=b.csv ..."""
import csv
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

dst, specs = sys.argv[1], sys.argv[2:]
fig, ax = plt.subplots(figsize=(14, 4.5))
for k, spec in enumerate(specs):
    label, path = spec.split("=", 1)
    rows = list(csv.DictReader(open(path)))
    t = [float(r["t"]) for r in rows]
    ax.plot(t, [float(r["observed"]) for r in rows], lw=1, label=f"{label} observed")
    if k == 0:
        ax.plot(t, [float(r["model"]) for r in rows], lw=1, alpha=0.7, label="model", color="k")
ax.set_xlabel("s from start")
ax.legend()
fig.tight_layout()
fig.savefig(dst, dpi=90)
print(dst)
