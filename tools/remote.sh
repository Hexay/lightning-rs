#!/usr/bin/env bash
# Run a heavy command on testbox (31 GB) instead of locally: sync repo + data, build release, run, pull results.
# Usage: [PULL="out/a out/b"] tools/remote.sh <command...>   e.g. tools/remote.sh bash tools/clips.sh tR6K
#        [PULL=...] tools/remote.sh --attach                 re-follow the last job after a dropped connection
# The job runs detached (nohup) and logs to ~/work/lightning-rs/remote.log, so a local kill or SSH drop
# doesn't stop it. Remote ffmpeg is a static build in ~/bin.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOST=testbox; DIR=work/lightning-rs
ssh() { command ssh -o ServerAliveInterval=30 -o ServerAliveCountMax=20 "$@"; }

if [ "${1:-}" != --attach ]; then
  tar -C "$ROOT" -cf - --exclude=./target --exclude=./out --exclude=./data/dem --exclude=./.git . \
    | ssh "$HOST" "mkdir -p $DIR && tar -C $DIR -xf -"
  job="export PATH=\$HOME/bin:\$HOME/.cargo/bin:\$PATH; cargo build --release -q 2>&1 | grep -E '^error' -A5; $(printf '%q ' "$@")"
  # `cd` must not share the backgrounded list, or remote.pid lands in ~.
  ssh "$HOST" "cd $DIR || exit 1; nohup bash -c $(printf '%q' "$job") > remote.log 2>&1 < /dev/null & echo \$! > remote.pid"
fi
ssh "$HOST" "cd $DIR && tail -n +1 -f --pid=\$(cat remote.pid) remote.log"
for p in ${PULL:-}; do
  ssh "$HOST" "tar -C $DIR -cf - $p" | tar -C "$ROOT" -xf -
done
