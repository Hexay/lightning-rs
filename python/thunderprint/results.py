"""Typed views of a `locate` output directory (candidates.json + trace.json)."""

from __future__ import annotations

import base64
import json
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path


def _utc(epoch: float) -> datetime:
    return datetime.fromtimestamp(epoch, tz=timezone.utc)


@dataclass(frozen=True)
class Region:
    """One candidate recording location. `rank` is 1-based, as in the viewer."""

    rank: int
    lat: float
    lon: float
    start: datetime
    probability: float
    rho: float
    cr90_area_km2: float
    reliable: bool
    audible_strikes: int
    ellipse_km: tuple[float, float]
    ellipse_bearing_deg: float
    speed_of_sound_mps: float
    wind_mps: tuple[float, float]


@dataclass(frozen=True)
class Spectrogram:
    """Log-frequency power, scaled 0–255 over the top 45 dB. `power[band][frame]`."""

    freqs_hz: list[float]
    offsets_s: list[float]
    power: list[list[int]]


@dataclass(frozen=True)
class Envelope:
    """Observed and modelled thunder envelopes (each scaled to unit max) for one region, sampled at `times`."""

    times: list[datetime]
    observed: list[float]
    modeled: list[float]


@dataclass
class Result:
    out_dir: Path
    regions: list[Region]
    spectrogram: Spectrogram
    _traces: dict

    @property
    def best(self) -> Region | None:
        return self.regions[0] if self.regions else None

    @property
    def viewer(self) -> Path:
        """Self-contained HTML map + trace panel; open it in a browser."""
        return self.out_dir / "index.html"

    @property
    def geopackage(self) -> Path:
        """GeoPackage (layers: cells, candidates, ellipses, onsets) for ArcGIS Pro / QGIS / geopandas."""
        return self.out_dir / "result.gpkg"

    def envelope(self, rank: int = 1) -> Envelope:
        tr = next(r for r in self._traces["regions"] if r["rank"] == rank)
        step = 1.0 / self._traces["rate_hz"]
        t0 = _utc(tr["start"])
        times = [t0 + timedelta(seconds=i * step) for i in range(len(self._traces["observed"]))]
        return Envelope(times, self._traces["observed"], tr["model"])

    def spectrogram_times(self, rank: int = 1) -> list[datetime]:
        t0 = self.regions[rank - 1].start
        return [t0 + timedelta(seconds=s) for s in self.spectrogram.offsets_s]


def load(out_dir: str | Path) -> Result:
    """Reads a finished `locate` output directory."""
    out_dir = Path(out_dir)
    cand = json.loads((out_dir / "candidates.json").read_text())
    trace = json.loads((out_dir / "trace.json").read_text())
    regions = [_region(r) for r in cand["regions"]]
    return Result(out_dir, regions, _spectrogram(trace["spectrogram"]), trace["traces"])


def _region(r: dict) -> Region:
    sc, sh, atm = r["score"], r["shape"], r["atmosphere"]
    return Region(
        rank=r["rank"] + 1,
        lat=r["map"]["lat"],
        lon=r["map"]["lon"],
        start=_utc(sc["best_start"]),
        probability=r["region_prob"],
        rho=sc["best_rho"],
        cr90_area_km2=r["cr90_area_km2"],
        reliable=r["reliable"],
        audible_strikes=sh["audible_strikes"],
        ellipse_km=(2 * sh["major_km"], 2 * sh["minor_km"]),
        ellipse_bearing_deg=sh["major_bearing_deg"],
        speed_of_sound_mps=atm["c_mps"],
        wind_mps=(atm["wind_e_mps"], atm["wind_n_mps"]),
    )


def _spectrogram(s: dict) -> Spectrogram:
    raw = base64.b64decode(s["data_b64"])
    n = s["frames"]
    power = [list(raw[k * n : (k + 1) * n]) for k in range(len(s["freqs_hz"]))]
    offsets = [s["t0_s"] + i * s["hop_s"] for i in range(n)]
    return Spectrogram(s["freqs_hz"], offsets, power)
