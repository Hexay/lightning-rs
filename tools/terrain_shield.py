"""Height (km above each strike's ground) hidden by terrain on the straight line from a listener.
Usage: py -3 -I tools/terrain_shield.py LAT LON residuals.csv [ear_m]"""
import csv
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # -I drops the script dir
from dem import Dem  # noqa: E402

KM_PER_DEG = 111.32
STEP_KM = 0.05


def hidden_km(dem, lat, lon, slat, slon, ear_m):
    kx = KM_PER_DEG * math.cos(math.radians(lat))
    de, dn = (slon - lon) * kx, (slat - lat) * KM_PER_DEG
    d = math.hypot(de, dn)
    z0 = dem.z(lat, lon) + ear_m
    zs = dem.z(slat, slon)
    slope = max(
        (dem.z(lat + dn * f / KM_PER_DEG, lon + de * f / kx) - z0) / (f * d * 1000)
        for f in (i * STEP_KM / d for i in range(1, int(d / STEP_KM)))
    )
    return d, max(0.0, (z0 + slope * d * 1000 - zs) / 1000), z0, zs


def main():
    lat, lon, path = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
    ear_m = float(sys.argv[4]) if len(sys.argv) > 4 else 2.0
    dem = Dem()
    with open(path) as f:
        rows = list(csv.DictReader(f))
    res = sorted(hidden_km(dem, lat, lon, float(r["lat"]), float(r["lon"]), ear_m) for r in rows)
    print(f"listener ground {res[0][2] - ear_m:.0f} m; {len(res)} strikes")
    for d, h, _, zs in res:
        print(f"  d {d:5.1f} km  strike ground {zs:4.0f} m  hidden to {h:.2f} km")
    hs = sorted(h for _, h, _, _ in res)
    print(f"hidden height median {hs[len(hs) // 2]:.2f} km, quartiles {hs[len(hs) // 4]:.2f}–{hs[3 * len(hs) // 4]:.2f}")


if __name__ == "__main__":
    main()
