#!/usr/bin/env bash
# Tabulate #1 error per sim case for finished suite runs, side by side (no re-running).
# Usage: tools/score_suite.sh [suffix ...]   e.g. tools/score_suite.sh "" -onset_grid
#   suffix "" = out/sim/<case>-improved, "-x" = out/sim/<case>-improved-x
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/target/release/thunderprint"
cd "$ROOT"
[ $# -eq 0 ] && set -- ""
printf '%-24s' case; for s in "$@"; do printf '%16s' "improved${s}"; done; echo
for d in data/sim/*/; do
  name=$(basename "$d"); [ -f "$d/truth.toml" ] || continue
  row=$(printf '%-24s' "$name"); any=0
  for s in "$@"; do
    run=out/sim/$name-improved$s
    if [ -f "$run/candidates.json" ]; then
      e=$("$B" eval --run "$run" --truth "$d/truth.toml" | grep -oE '^#1 error [0-9.]+ km' | grep -oE '[0-9.]+'); any=1
      rel=$("$B" eval --run "$run" --truth "$d/truth.toml" | grep -q '^shape:.*UNRELIABLE' && echo '*' || echo ' ')
      row+=$(printf '%15s%s' "$e" "$rel")
    else row+=$(printf '%16s' -); fi
  done
  [ $any = 1 ] && echo "$row"
done
echo "(* = flagged UNRELIABLE)"
