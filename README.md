# lightning-rs

Locate where (and when) an audio recording was made from the thunder in it, by matching it against
lightning detections from MET Norway's Frost API.

## Setup

- Rust (stable), `ffmpeg` on PATH.
- `.env` in the repo root with `FROST_CLIENT_ID=...` (free at frost.met.no).

## Usage

```
cargo build --release
B=target/release/lightning-rs

$B locate --audio clip.opus --date 2025-07-19 --out out/run          # improved model
$B locate --audio clip.opus --date 2025-07-19 --model baseline ...    # reference method
$B eval --run out/run --truth eval/truth.toml                         # score against held-out truth
$B sim --date 2025-07-19 --lat .. --lon .. --start 15:10:00 --out data/sim/x   # synthetic test clip
$B probe --audio .. --date .. --lat .. --lon .. [--start EPOCH] [--bench]       # diagnostics
```

`locate` writes `heatmap.geojson`, `candidates.json` and a self-contained `index.html` map viewer.
Search settings can be overridden with `--overrides file.toml` (fields in `src/config.rs`).

`tools/sim_suite.sh` runs the synthetic accuracy suite; `tools/probe.sh` plots model vs observed at a sim's truth.
