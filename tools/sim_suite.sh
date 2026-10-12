#!/usr/bin/env bash
# Synthetic accuracy suite on 2025-07-19 storms away from the real test region.
# Usage: [OVR=overrides.toml] tools/sim_suite.sh [improved|baseline|both] [case-filter]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/lightning-rs"
MODELS="${1:-both}"; FILTER="${2:-}"
[ "$MODELS" = both ] && MODELS="improved baseline"

# name lat lon start temp wind_e wind_n noise seed
CASES=(
  "inside_cluster 62.25 8.25 15:10:00 14 3 -2 0.02 1"
  "east_storm 62.25 9.75 18:30:00 12 -5 1 0.03 2"
  "west_coast_se 58.25 11.75 13:20:00 18 2 4 0.02 3"
  "cluster_to_side 62.55 8.70 14:30:00 10 6 0 0.04 4"
)

for c in "${CASES[@]}"; do
  read -r name lat lon start temp we wn noise seed <<<"$c"
  [[ -n "$FILTER" && "$name" != *"$FILTER"* ]] && continue
  dir="$ROOT/data/sim/$name"
  [ -f "$dir/audio.wav" ] || (cd "$ROOT" && "$BIN" sim --date 2025-07-19 --lat "$lat" --lon "$lon" --start "$start" \
      --temp="$temp" --wind-e="$we" --wind-n="$wn" --noise "$noise" --seed "$seed" --out "$dir" >/dev/null)
  for m in $MODELS; do
    out="$ROOT/out/sim/$name-$m${OVR:+-$(basename "$OVR" .toml)}"
    mkdir -p "$ROOT/out/sim"
    (cd "$ROOT" && "$BIN" locate --audio "$dir/audio.wav" --date 2025-07-19 --model "$m" --out "$out" ${OVR:+--overrides "$OVR"} >"$out.txt" 2>"$out.log") || { echo "$name $m FAILED"; tail -5 "$out.log"; continue; }
    echo "== $name [$m] $(grep -o 'in [0-9.]*s total' "$out.log")"
    (cd "$ROOT" && "$BIN" eval --run "$out" --truth "$dir/truth.toml" | grep -E "^#1 |^#2 |credible|^onset solve|^coarse:|^mid:" | grep -vE "^onset solve error [0-9]{3}" | head -9)
  done
done
