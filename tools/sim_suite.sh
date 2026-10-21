#!/usr/bin/env bash
# Synthetic accuracy suite on 2025-07-19 storms away from the real test region.
# Usage: [OVR=overrides.toml] [MICS="spacing_m axis_deg"] tools/sim_suite.sh [improved|baseline|both] [case-filter]
# MICS synthesises a two-mic (stereo) recording into data/sim/<case>-stereo.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/thunderprint"
MODELS="${1:-both}"; FILTER="${2:-}"
[ "$MODELS" = both ] && MODELS="improved baseline"

# name lat lon start temp wind_e wind_n noise seed
# side_*: listener ~15 km off a storm core (like the real clip); the next four: inside/next to dense storms.
# Apart from real_twin these hear ≤ 21 strikes at the found spot: they test the reliability flag, not accuracy.
CASES=(
  "real_twin 65.666899 14.138535 15:59:51 14 0 0 0.02 7"
  "side_e_of_8E 62.25 8.55 15:10:00 14 2 -3 0.03 11"
  "side_n_of_west_coast 58.40 11.75 13:20:00 18 -3 2 0.03 12"
  "side_w_of_19E 65.25 18.90 14:30:00 12 4 4 0.03 13"
  "inside_cluster 62.25 8.25 15:10:00 14 3 -2 0.02 1"
  "east_storm 62.25 9.75 18:30:00 12 -5 1 0.03 2"
  "west_coast_se 58.25 11.75 13:20:00 18 2 4 0.02 3"
  "cluster_to_side 62.55 8.70 14:30:00 10 6 0 0.04 4"
  # loc_*: locatable tier from tools/pick_sim_cases.py (≥ 88 strikes heard); *_wide = storm around the listener.
  "loc_side_58N 58.408 11.501 13:12:10 18 -3 2 0.03 21"
  "loc_wide_62N8E 62.385 8.075 15:18:20 14 3 -2 0.03 22"
  "loc_wide_62N9E 62.309 8.615 14:17:50 12 0 4 0.02 23"
  "loc_side_65N16E 64.566 15.700 14:17:10 16 -4 -1 0.03 24"
  "loc_wide_65N19E 65.089 19.339 14:40:30 12 4 4 0.03 25"
  "loc_side_65N19E 64.628 19.432 13:25:20 15 2 -3 0.02 26"
  "loc_wide_62N10E 62.238 9.567 17:56:10 12 -5 1 0.03 27"
)

for c in "${CASES[@]}"; do
  read -r name lat lon start temp we wn noise seed <<<"$c"
  [[ -n "$FILTER" && "$name" != *"$FILTER"* ]] && continue
  read -r spacing axis <<<"${MICS:-0 0}"
  [ "$spacing" != 0 ] && name="$name-stereo"
  dir="$ROOT/data/sim/$name"
  [ -f "$dir/audio.wav" ] || (cd "$ROOT" && "$BIN" sim --date 2025-07-19 --lat "$lat" --lon "$lon" --start "$start" \
      --temp="$temp" --wind-e="$we" --wind-n="$wn" --noise "$noise" --seed "$seed" \
      --mic-spacing "$spacing" --mic-axis="$axis" --out "$dir" >/dev/null)
  for m in $MODELS; do
    out="$ROOT/out/sim/$name-$m${OVR:+-$(basename "$OVR" .toml)}"
    mkdir -p "$ROOT/out/sim"
    (cd "$ROOT" && "$BIN" locate --audio "$dir/audio.wav" --date 2025-07-19 --model "$m" --out "$out" ${OVR:+--overrides "$OVR"} >"$out.txt" 2>"$out.log") || { echo "$name $m FAILED"; tail -5 "$out.log"; continue; }
    echo "== $name [$m] $(grep -o 'in [0-9.]*s total' "$out.log")"
    (cd "$ROOT" && "$BIN" eval --run "$out" --truth "$dir/truth.toml" | grep -E "^#1 |^#2 |credible|^onset solve|^coarse:|^mid:" | grep -vE "^onset solve error [0-9]{3}" | head -9)
  done
done
