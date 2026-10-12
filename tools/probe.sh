#!/usr/bin/env bash
# Probe a sim case at its true location/start and plot model vs observed.
# Usage: tools/probe.sh <case> [overrides.toml]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
dir="$ROOT/data/sim/$1"
lat=$(grep ^lat "$dir/truth.toml" | cut -d' ' -f3)
lon=$(grep ^lon "$dir/truth.toml" | cut -d' ' -f3)
start=$(grep ^start "$dir/truth.toml" | cut -d' ' -f3)
ovr=()
[ -n "${2:-}" ] && ovr=(--overrides "$2")
cd "$ROOT"
"$ROOT/target/release/lightning-rs" probe --audio "$dir/audio.wav" --date 2025-07-19 --lat "$lat" --lon "$lon" --start "$start" "${ovr[@]}"
py -3 -I "$ROOT/tools/plot_probe.py" out/probe.csv "out/probe-$1.png" >/dev/null
