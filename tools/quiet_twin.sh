#!/usr/bin/env bash
# Does a quiet lower CG channel in the sim reproduce the real clip's failure (wrong pick ~2.6 km S)?
# Usage: tools/quiet_twin.sh "3 5" "7 8"   (quiet-below heights km, seeds)
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/target/release/thunderprint"; E="$ROOT/experiments"
cd "$ROOT"
for q in ${1:-3 5}; do for s in ${2:-7}; do
  d=data/sim/real_twin_quiet${q}_s$s
  [ -f $d/audio.wav ] || $B sim --date 2025-07-19 --lat 65.666899 --lon 14.138535 --start 15:59:51 \
      --temp=14 --noise 0.02 --seed $s --cg-quiet-below $q --out $d >/dev/null
  for ov in "" cg_onset_4 cg_onset_6; do
    for site in truth:65.666899:14.138535 wrong:65.64379:14.12508; do IFS=: read -r sn la lo <<<"$site"
      ev=$($B probe --audio $d/audio.wav --date 2025-07-19 --lat $la --lon $lo ${ov:+--overrides $E/$ov.toml} | grep -E '^ *fine:' | grep -oE 'log_ev +[-0-9.]+')
      echo "quiet$q s$s ${ov:-h0} $sn $ev"
    done
    out=out/refr/quiet${q}_s$s-${ov:-h0}
    $B locate --audio $d/audio.wav --date 2025-07-19 ${ov:+--overrides $E/$ov.toml} --out $out >/dev/null 2>$out.log
    echo "  locate: $($B eval --run $out --truth $d/truth.toml | grep -E '^#1 ' | cut -c1-70)"
  done
done; done
