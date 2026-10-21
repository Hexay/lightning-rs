#!/usr/bin/env bash
# Locate the extra real clips (data/clips, see research/real_clip_candidates.md) and score against
# their claimed locations. Usage: [OVR=overrides.toml] tools/clips.sh [id-filter]
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/target/release/lightning-rs"
cd "$ROOT"
# id date lat lon
CLIPS=(
  "QdOqeb2Adv0 2020-08-13 59.9177 10.7353"
  "cA1jXQOOfjY 2015-08-07 63.36 10.38"
  "SQTYRo0b__s 2021-07-30 60.14 11.17"
  "tR6K-m_5kVw 2014-08-04 59.95 12.93"
  "_cvFqOvDH9E 2014-07-24 59.99 8.76"
  "hZWZlNOZhyY 2018-08-01 69.49 20.83"
)
mkdir -p eval/clips out/clips
for c in "${CLIPS[@]}"; do
  read -r id date lat lon <<<"$c"
  [[ -n "${1:-}" && "$id" != *"$1"* ]] && continue
  printf 'lat = %s\nlon = %s\n' "$lat" "$lon" > eval/clips/$id.toml
  out=out/clips/$id${OVR:+-$(basename "$OVR" .toml)}
  $B locate --audio data/clips/$id.webm --date $date ${OVR:+--overrides "$OVR"} --out $out >/dev/null 2>$out.log \
    || { echo "== $id FAILED: $(tail -2 $out.log)"; continue; }
  echo "== $id $date $(grep -oE '[0-9.]+ s audio|in [0-9.]*s total' $out.log | tr '\n' ' ')"
  $B eval --run $out --truth eval/clips/$id.toml | grep -E "^#1 |^#2 |credible" | head -3
done
