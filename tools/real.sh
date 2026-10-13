#!/usr/bin/env bash
# Locate the real vid2 clip and score it against eval/truth.toml.
# Usage: tools/real.sh [overrides.toml ...]   (no args = defaults)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/lightning-rs"
run() {
  local name="$1"; shift
  local out="$ROOT/out/real-$name"
  (cd "$ROOT" && "$BIN" locate --audio data/audio/thunderstorm.opus --date 2025-07-19 "$@" --out "$out" >/dev/null 2>"$out.log")
  echo "== real [$name] $(grep -o 'in [0-9.]*s total' "$out.log")"
  (cd "$ROOT" && "$BIN" eval --run "$out" --truth eval/truth.toml | grep -E "^#1 |^#2 |credible|^coarse:")
}
[ $# -eq 0 ] && run default
for ovr in "$@"; do run "$(basename "$ovr" .toml)" --overrides "$ovr"; done
