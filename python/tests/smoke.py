"""End-to-end check of an installed wheel, offline: synthetic strikes -> simulate -> locate -> results.

    python python/tests/smoke.py
"""

import math
import random
import sys
import tempfile
from datetime import datetime, timedelta
from pathlib import Path

import thunderprint as tp

DATE, START = "2025-07-19", "14:40:30"
LAT, LON = 65.089, 19.339


def synthetic_strikes(n: int = 80, seed: int = 1) -> list[dict]:
    rng = random.Random(seed)
    t0 = datetime.fromisoformat(f"{DATE}T{START}")
    rows = []
    for _ in range(n):
        r_km, theta = rng.uniform(2, 18), rng.uniform(0, 2 * math.pi)
        rows.append({
            "time": t0 + timedelta(seconds=rng.uniform(-30, 270)),
            "lat": LAT + r_km * math.cos(theta) / 111.2,
            "lon": LON + r_km * math.sin(theta) / (111.2 * math.cos(math.radians(LAT))),
            "peak_ka": rng.uniform(5, 60),
        })
    return rows


def main() -> int:
    work = Path(tempfile.mkdtemp(prefix="thunderprint-smoke-"))
    strikes = synthetic_strikes()
    wav = tp.simulate(DATE, LAT, LON, START, work / "sim", strikes=strikes, noise=0.02, seed=1)
    res = tp.locate(wav, DATE, start=START, strikes=strikes, out_dir=work / "run")

    b = res.best
    km = math.hypot((b.lat - LAT) * 111.2, (b.lon - LON) * 111.2 * math.cos(math.radians(LAT)))
    print(f"best #{b.rank} {b.lat:.4f},{b.lon:.4f} err {km:.2f} km reliable {b.reliable}")
    assert km < 3, f"located {km:.2f} km from truth"
    assert res.viewer.exists() and res.geopackage.exists()
    assert len(res.envelope().times) > 0

    try:
        tp.locate(work / "missing.wav", DATE, start=START, strikes=strikes, out_dir=work / "bad")
    except tp.ThunderprintError:
        pass
    else:
        raise AssertionError("missing audio did not raise ThunderprintError")
    print("ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
