#!/usr/bin/env bash
# Refraction matrix: twin sims with/without a lapse-rate shadow, and the real clip, under lapse overrides.
# Usage: tools/refraction.sh   (results in out/refr/*.log; summary on stdout)
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/target/release/lightning-rs"; E="$ROOT/experiments"
cd "$ROOT"
for l in 6.5 9.8; do
  d=data/sim/real_twin_lapse$l
  [ -f $d/audio.wav ] || $B sim --date 2025-07-19 --lat 65.666899 --lon 14.138535 --start 15:59:51 \
      --temp=14 --noise 0.02 --seed 7 --lapse $l --out $d >/dev/null
done
loc() { # name audio truth [overrides]
  local out=out/refr/$1; mkdir -p out/refr
  $B locate --audio "$2" --date 2025-07-19 ${4:+--overrides "$4"} --out $out >/dev/null 2>$out.log
  echo "== $1 $(grep -o 'in [0-9.]*s total' $out.log)"
  $B eval --run $out --truth "$3" | grep -E "^#1 |credible" | head -3
}
for tw in real_twin_lapse6.5 real_twin_lapse9.8; do
  t=data/sim/$tw/truth.toml; a=data/sim/$tw/audio.wav
  loc $tw-default $a $t
  loc $tw-l6.5 $a $t $E/lapse_6.5.toml
  loc $tw-grid $a $t $E/lapse_grid.toml
done
loc twin-grid data/sim/real_twin/audio.wav data/sim/real_twin/truth.toml $E/lapse_grid.toml
loc twin-l6.5 data/sim/real_twin/audio.wav data/sim/real_twin/truth.toml $E/lapse_6.5.toml
for o in lapse_6.5 lapse_9.8 lapse_grid; do loc real-$o data/audio/thunderstorm.opus eval/truth.toml $E/$o.toml; done
loc real-default data/audio/thunderstorm.opus eval/truth.toml
