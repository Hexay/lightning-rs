"""Plot out/probe.csv (observed vs model envelope) to out/probe.png."""
import csv
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

src = sys.argv[1] if len(sys.argv) > 1 else "out/probe.csv"
dst = sys.argv[2] if len(sys.argv) > 2 else "out/probe.png"
rows = list(csv.DictReader(open(src)))
t = [float(r["t"]) for r in rows]
fig, ax = plt.subplots(figsize=(14, 4))
ax.plot(t, [float(r["observed"]) for r in rows], label="observed", lw=1)
ax.plot(t, [float(r["model"]) for r in rows], label="model", lw=1, alpha=0.8)
ax.set_xlabel("s from start")
ax.legend()
fig.tight_layout()
fig.savefig(dst, dpi=90)
print(dst)
