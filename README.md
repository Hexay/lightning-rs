# lightning-rs

[![CI](https://github.com/Hexay/lightning-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexay/lightning-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Work out where and when an audio recording was made using only the thunder in it.

Lightning networks log every strike with millisecond timing and positions accurate to a few hundred metres.
Each strike's thunder reaches a listener after a delay set by distance. A recording of a storm therefore
carries the listener's position, as the delay pattern across dozens of strikes. lightning-rs searches a
day of strikes from [MET Norway's Frost API](https://frost.met.no) for the place and start time whose
predicted thunder best matches the recording. It outputs a probability map with a 90% credible region.

The output is a self-contained HTML map: a heatmap, ranked candidate regions and their 90% ellipses.

## Install

Requirements:

- Rust (stable, edition 2024)
- [`ffmpeg`](https://ffmpeg.org) on `PATH` to decode audio. Any format ffmpeg reads works.
- A free Frost client ID from [frost.met.no](https://frost.met.no/howto.html), in a `.env` file at the repo root:

  ```
  FROST_CLIENT_ID=your-client-id
  ```

Then build:

```sh
git clone https://github.com/Hexay/lightning-rs
cd lightning-rs
cargo build --release
```

## Quickstart

You don't need a real recording to try it. `sim` synthesises thunder from real strikes at a listener you
choose, and you can then check whether `locate` finds that listener:

```sh
B=target/release/lightning-rs

$B sim --date 2025-07-19 --lat 65.089 --lon 19.339 --start 14:40:30 \
  --temp=12 --wind-e=4 --wind-n=4 --noise 0.03 --seed 25 --out data/sim/demo
$B locate --audio data/sim/demo/audio.wav --date 2025-07-19 --out out/demo
$B eval --run out/demo --truth data/sim/demo/truth.toml
```

`eval` should report an error of about 0.4 km. Open `out/demo/index.html` to see the map, with modeled and
observed thunder and a spectrogram underneath it. Lightning for each UTC day is downloaded once and cached in `data/frost/`.

## Usage

| Command | What it does |
|---|---|
| `locate --audio F --date D [--start HH:MM:SS]` | Searches every strike on UTC day `D` for where and when `F` was recorded. If you know the start time, `--start` narrows the search to ±0.5 s. |
| `eval --run DIR --truth T` | Scores a finished run against a truth file (`lat`/`lon` TOML). |
| `sim --date D --lat .. --lon .. --start ..` | Writes a synthetic `audio.wav` and `truth.toml`. Run `--help` for noise, wind, stereo and propagation options. |
| `fetch --date D` | Downloads and caches one day of Frost lightning ahead of time. |
| `probe ...` | Diagnostics: scores one point at every pipeline stage. |

`locate` writes three files to `--out` (default `out/run`):

- `index.html`: the map viewer. Under the map, it plots the modeled and observed thunder envelopes over UTC time, with a 10–1000 Hz spectrogram on the same time axis. Clicking a candidate region switches the modeled curve to that region.
- `heatmap.geojson`: every scored grid cell
- `candidates.json`: ranked regions, each with its start time, fitted atmosphere, 90% area and a `reliable` flag

### Models and overrides

- **`--model improved`** (the default) is the model described below.
- **`--model baseline`** reimplements the method from the video that inspired this project (see [Prior art](#prior-art)). It's there for comparison.
- **`--overrides file.toml`** overrides search settings. The fields are declared in [`src/config.rs`](src/config.rs) and [`experiments/`](experiments) has examples.

## How it works

1. **Observed envelope.** The audio is band-passed to 20–250 Hz, where thunder carries far and through walls. It's then reduced to a smoothed, compressed loudness envelope.
2. **Predicted envelope.** For a candidate listener position, every strike is rendered as a thunder pulse.
   - Cloud-to-ground strikes are modelled as extended vertical channels, intra-cloud strikes as horizontal sources aloft.
   - Arrival time is slant distance divided by the speed of sound, corrected for wind.
   - Loudness scales with peak current and falls off with range.
   - Each pulse has a rumble tail whose length grows with the channel's spread of arrival times.
3. **Match.** The predicted envelope is correlated with the observed one at every possible start time in the day, using FFT cross-correlation.
4. **Search, coarse to fine.** A coarse grid covers the whole strike region, then mid and fine grids refine the best peaks.
   - The fine stage marginalises over nuisance atmospheres: temperature (speed of sound), wind, refraction lapse rate and the effective height thunder onsets are timed from.
5. **Posterior.** Scores are tempered into a posterior, from which the tool reports ranked regions and 90% credible ellipses. A region is marked unreliable when fewer than 30 strikes would have been audible from it.

Two physical findings drove the model. Both are written up in [`research/`](research):

- **CG onset height.** Real thunder onsets behave as if they come from about 6 km up, not from the channel base (see [empirical_thunder_response.md](research/empirical_thunder_response.md)).
- **Refraction shadow.** Upward refraction hides the low part of distant channels (see [thunder_physics_lit.md](research/thunder_physics_lit.md)).

## Accuracy and limitations

| Case | Error |
|---|---|
| Synthetic clips, listener near active storms | 0.4–1.0 km |
| Synthetic twin of the real clip | 1.4 km |
| Real 4-minute recording, northern Norway | 1.3 km, versus 2.2 km for the baseline method. Not blind: the truth was known during development. |

Limitations:

- **Coverage.** Frost lightning covers the Nordic detection network (Norway and neighbouring countries) and nothing else.
- **Recording length.** You need several minutes of unedited audio recorded inside an active storm. Short or edited clips don't hear enough strikes to fix a position.
- **Geometry.** If every strike comes from one direction, the result is a stretched ellipse. Strikes from several directions give a tight fix.
- **Video clocks.** In a video, the visible flashes give the frame clock. The audio track can be seconds away from that, so `--start` taken from flashes needs care.
- **Real clips fit worse than simulations.** The model is still incomplete, and the credible regions on real audio should be read as optimistic.

## Prior art

- The baseline method comes from colsto's videos [*Can you geolocate a video using background thunder sounds?*](https://www.youtube.com/watch?v=SnbkIm56TfI) and [*I located a fan's video using thunder sounds*](https://www.youtube.com/watch?v=6a7Z0I3LAnE). The method is a point source, 343 m/s, inverse-square attenuation and cosine similarity.
- Related work on timing and locating recordings is surveyed in [`research/prior_art_receiver_location.md`](research/prior_art_receiver_location.md).

## Data and attribution

- **Lightning data:** from [MET Norway](https://www.met.no) via the [Frost API](https://frost.met.no), licensed under [NLOD 2.0 / CC BY 4.0](https://www.met.no/en/free-meteorological-data/Licensing-and-crediting). Raw strike files are not redistributed here; each user downloads them with their own client ID.
- **Map tiles in the viewer:** © Esri, HERE, Garmin and © OpenStreetMap contributors.

## License

[MIT](LICENSE)
