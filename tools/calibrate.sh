#!/usr/bin/env bash
# Sweep n_eff_factor over the sim suite (+ real clip as a non-blind check); prints error and credible level.
# Usage: tools/calibrate.sh 0.5 0.2 0.1 ...
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/thunderprint"
mkdir -p "$ROOT/experiments" "$ROOT/out"
for f in "$@"; do
  ovr="$ROOT/experiments/neff_$f.toml"
  echo "n_eff_factor = $f" > "$ovr"
  echo "######## n_eff_factor $f"
  OVR="$ovr" bash "$ROOT/tools/sim_suite.sh" improved | grep -E "^==|^#1 |credible"
  out="$ROOT/out/real-neff_$f"
  (cd "$ROOT" && "$BIN" locate --audio data/audio/thunderstorm.opus --date 2025-07-19 --overrides "$ovr" --out "$out" >/dev/null 2>&1)
  echo "== real"
  (cd "$ROOT" && "$BIN" eval --run "$out" --truth eval/truth.toml | grep -E "^#1 |credible")
done
