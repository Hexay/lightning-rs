"""Locate and run the bundled `thunderprint` binary."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import sysconfig
from pathlib import Path

EXE = "thunderprint" + (".exe" if sys.platform == "win32" else "")


class ThunderprintError(RuntimeError):
    """The thunderprint binary exited with an error; `stderr` holds its message."""

    def __init__(self, args: list[str], stderr: str):
        super().__init__(f"thunderprint {' '.join(args[:1])} failed:\n{stderr.strip()}")
        self.stderr = stderr


def binary() -> str:
    # The wheel installs the binary into this interpreter's scripts dir, which may not be on PATH (e.g. --user installs).
    for d in (sysconfig.get_path("scripts"), sysconfig.get_path("scripts", f"{os.name}_user")):
        if d and (Path(d) / EXE).is_file():
            return str(Path(d) / EXE)
    found = shutil.which("thunderprint")
    if not found:
        raise FileNotFoundError("thunderprint binary not found; reinstall the package or put it on PATH")
    return found


def default_cache() -> Path:
    if sys.platform == "win32":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Caches"
    else:
        base = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return base / "thunderprint" / "frost"


def run(args: list[str], frost_client_id: str | None = None, quiet: bool = True) -> str:
    """Runs the binary with `args`; returns stdout. Progress (stderr) is shown unless `quiet`."""
    env = dict(os.environ)
    env.setdefault("THUNDERPRINT_CACHE", str(default_cache()))
    if frost_client_id:
        env["FROST_CLIENT_ID"] = frost_client_id
    proc = subprocess.run([binary(), *args], env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE if quiet else None)
    if proc.returncode != 0:
        raise ThunderprintError(args, proc.stderr or "")
    return proc.stdout
