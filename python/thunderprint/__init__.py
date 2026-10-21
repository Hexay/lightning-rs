"""Locate where and when a recording was made from the thunder in it.

    import thunderprint as tp
    res = tp.locate("storm.opus", "2025-07-19")
    print(res.best)          # Region(rank=1, lat=..., lon=..., start=..., reliable=True, ...)
    env = res.envelope()     # observed vs modelled thunder over UTC time
    res.viewer               # path to the interactive HTML map

Lightning comes from MET Norway's Frost API (Nordic coverage; set FROST_CLIENT_ID or pass
`frost_client_id`), or from your own strikes via `strikes=` (a CSV path, or rows / a DataFrame
with time, lat, lon[, peak_ka, cloud, error_km]).
"""

from __future__ import annotations

import csv
import tempfile
from datetime import date as _date, datetime, time as _time, timezone
from pathlib import Path
from typing import Iterable, Mapping

from ._cli import ThunderprintError, run
from .results import Envelope, Region, Result, Spectrogram, load

__all__ = ["locate", "simulate", "fetch", "load", "Result", "Region", "Envelope", "Spectrogram", "ThunderprintError"]

STRIKE_COLUMNS = ("time", "lat", "lon", "peak_ka", "cloud", "error_km")


def locate(
    audio: str | Path,
    date: str | _date,
    *,
    start: str | _time | None = None,
    strikes: str | Path | Iterable[Mapping] | None = None,
    out_dir: str | Path | None = None,
    model: str = "improved",
    overrides: str | Path | None = None,
    frost_client_id: str | None = None,
    quiet: bool = True,
) -> Result:
    """Searches every strike on UTC `date` for where and when `audio` was recorded.

    `start` (UTC time of day) narrows the search to ±0.5 s when the recording start is known.
    Results are written to `out_dir` (a temp dir if omitted) and returned parsed. Takes minutes.
    """
    out = Path(out_dir) if out_dir else Path(tempfile.mkdtemp(prefix="thunderprint-"))
    with _strike_file(strikes) as strikes_path:
        args = ["locate", "--audio", str(audio), "--date", _iso_date(date), "--model", model, "--out", str(out)]
        args += _opt("--start", start and _iso_time(start)) + _opt("--overrides", overrides) + _opt("--strikes", strikes_path)
        run(args, frost_client_id, quiet)
    return load(out)


def simulate(
    date: str | _date,
    lat: float,
    lon: float,
    start: str | _time,
    out_dir: str | Path,
    *,
    strikes: str | Path | Iterable[Mapping] | None = None,
    frost_client_id: str | None = None,
    **options: float,
) -> Path:
    """Synthesises a thunder recording at (`lat`, `lon`) from real strikes; returns the audio.wav path.

    `options` map to `thunderprint sim` flags, e.g. duration=260, temp=14, wind_e=0, noise=0.02, seed=1.
    """
    out = Path(out_dir)
    with _strike_file(strikes) as strikes_path:
        args = ["sim", "--date", _iso_date(date), f"--lat={lat}", f"--lon={lon}", "--start", _iso_time(start), "--out", str(out)]
        args += [f"--{k.replace('_', '-')}={v}" for k, v in options.items()] + _opt("--strikes", strikes_path)
        run(args, frost_client_id)
    return out / "audio.wav"


def fetch(date: str | _date, *, frost_client_id: str | None = None) -> int:
    """Downloads and caches one UTC day of Frost lightning; returns the strike count."""
    return int(run(["fetch", "--date", _iso_date(date)], frost_client_id).split()[0])


def _opt(flag: str, value) -> list[str]:
    return [flag, str(value)] if value else []


def _iso_date(d: str | _date) -> str:
    return d.isoformat() if isinstance(d, _date) else d


def _iso_time(t: str | _time) -> str:
    return t.isoformat() if isinstance(t, _time) else t


class _strike_file:
    """Context manager: a CSV path for `strikes`, writing rows / a DataFrame to a temp file if needed."""

    def __init__(self, strikes):
        self.strikes, self.tmp = strikes, None

    def __enter__(self) -> str | None:
        s = self.strikes
        if s is None or isinstance(s, (str, Path)):
            return s and str(s)
        rows = s.to_dict("records") if hasattr(s, "to_dict") else list(s)
        self.tmp = tempfile.NamedTemporaryFile("w", suffix=".csv", delete=False, newline="")
        cols = [c for c in STRIKE_COLUMNS if rows and c in rows[0]]
        w = csv.writer(self.tmp)
        w.writerow(cols)
        for r in rows:
            w.writerow([_csv_value(r[c]) for c in cols])
        self.tmp.close()
        return self.tmp.name

    def __exit__(self, *exc):
        if self.tmp:
            Path(self.tmp.name).unlink(missing_ok=True)


def _csv_value(v) -> str:
    if isinstance(v, datetime):
        # Naive datetimes are UTC here; .timestamp() alone would read them as local time.
        return f"{(v if v.tzinfo else v.replace(tzinfo=timezone.utc)).timestamp():.6f}"
    if isinstance(v, bool):
        return "1" if v else "0"
    return str(v)
