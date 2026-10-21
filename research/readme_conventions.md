# README & repo-hygiene conventions for public release

Surveyed 2026-10-20 (web only).

## Repos studied

| Repo | Domain | License | CITATION.cff | CONTRIBUTING | CI |
|---|---|---|---|---|---|
| [bellingcat/ShadowFinder](https://github.com/bellingcat/ShadowFinder) | OSINT chronolocation/geolocation | MIT | no | README section + CoC | Actions, pre-commit, tox |
| [RaphaelJ/libhum](https://github.com/RaphaelJ/libhum) | ENF audio forensics (timestamping) | LGPL-3.0 | yes (+ BibTeX in README) | no | no |
| [LCAV/pyroomacoustics](https://github.com/LCAV/pyroomacoustics) | acoustic source localization | MIT | no (paper cited in README) | CONTRIBUTING.rst, PR template | Actions, RTD |
| [obspy/obspy](https://github.com/obspy/obspy) | seismology / geophysics | LGPL-3.0 | Zenodo concept DOI | CONTRIBUTING.md, CoC, CODEOWNERS | Actions, codecov |
| [deeplycloudy/xlma-python](https://github.com/deeplycloudy/xlma-python) | lightning mapping array analysis | MIT | no | no | Actions |
| [sharkdp/hyperfine](https://github.com/sharkdp/hyperfine) | small Rust CLI | MIT OR Apache-2.0 | yes | no | Actions |
| [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | Rust CLI reference | Unlicense OR MIT | no | FAQ/GUIDE docs | Actions, ISSUE_TEMPLATE/ |

## 1. README section ordering

Most repos follow roughly this order:
**title + badges → one-paragraph pitch → hero figure → install → usage/quickstart → how it works → (limitations) → contributing → citation → license.**

- **Pitch and figure first.** ShadowFinder opens with one sentence ("given object height, shadow length, date/time → candidate locations") followed by its output map. hyperfine puts a demo GIF straight under the title. For thunderprint, the obvious hero is the map with the 90% ellipse against the true location.
- **Zero-setup quickstart.** ShadowFinder offers a Colab notebook plus `pip install` and separate Library and CLI usage sections. pyroomacoustics has "Quick Install" followed by a runnable "Example". hyperfine starts with a one-liner under "Basic benchmarks". For thunderprint: `cargo install`, then a single command on a bundled sample clip, with the Frost client-ID setup shown first.
- **How it works.** Only the scientific repos have this section. xlma-python calls it "Technical architecture" (data model → analysis → display), and pyroomacoustics has "Summary". Keep it to a short pipeline (onset detection → strike association → grid/probe search) and link to docs/ for the detail.
- **Accuracy/limitations.** None of the surveyed repos had a dedicated section. libhum gives only a speed claim, and ShadowFinder's only caveat is in a CLI note. This would set thunderprint apart. Include measured error on the sim suite and on real clips, and state the failure modes plainly: short or edited clips can't be located, it needs many audible strikes, and the geometry matters.
- **Prior art / alternatives.** xlma-python ("Prior art") and hyperfine ("Alternative tools") both have one. A short paragraph citing ENF chronolocation, shadow geolocation and thunder ranging fits here.
- **Citation then License close the README** (hyperfine, libhum, pyroomacoustics). libhum puts BibTeX in the README and also ships CITATION.cff.

## 2. Hygiene files

- **LICENSE:** The Rust convention is dual `LICENSE-MIT` + `LICENSE-APACHE` with `license = "MIT OR Apache-2.0"` (hyperfine). Scientific Python tools use MIT, or LGPL-3.0 for copyleft. Recommended: MIT OR Apache-2.0.
- **CITATION.cff:** Shipped by hyperfine and libhum. GitHub renders a "Cite this repository" button from it. Mint a Zenodo DOI on the first release, as ObsPy does.
- **CONTRIBUTING / CoC:** Only the larger projects have these. For a solo research tool, a short "Contributing" README section is enough.
- **CI:** Every repo except libhum has GitHub Actions. Minimum for thunderprint: `cargo fmt --check`, `clippy -D warnings`, and `cargo test` on Linux and Windows.
- **Issue templates:** ripgrep has `.github/ISSUE_TEMPLATE/`. Optional here; a bug template that asks for the clip length and the region would help.
- **Cargo.toml metadata** (from ripgrep/hyperfine): `description`, `license`, `repository`, `homepage`, `readme`, `keywords` (max 5, e.g. `lightning`, `thunder`, `geolocation`, `osint`, `acoustics`), `categories` (`command-line-utilities`, `science::geo`, `multimedia::audio`), `rust-version`, and `exclude` to drop `/research`, `/eval`, `/experiments` and `/tools` from the crate.

## 3. MET Norway / Frost data terms

- **Licence:** "Unless specified otherwise, all data and products are licensed under" **NLOD 2.0 and CC BY 4.0** ([docs.api.met.no/doc/License](https://docs.api.met.no/doc/License), [met.no Licensing and crediting](https://www.met.no/en/free-meteorological-data/Licensing-and-crediting)).
- **Required attribution:** credit "The Norwegian Meteorological Institute (MET Norway)". The suggested wording is «Data from MET Norway» or «Based on data from MET Norway», and they also ask for a link to where the data can be downloaded. Add a "Data sources & attribution" README section and print the same credit line in the viewer/map footer.
- **Frost terms** ([frost.met.no/termsofuse2.html](https://frost.met.no/termsofuse2.html)) say the licence "is found in every response". JSON responses carry a licence field, but whether UALF lightning responses do is unverified. Other terms: "do not ask too often", cache locally above about 1 request/s, and keep the client secret private. Don't commit credentials; read them from an env var or config.
- **Caveat:** MET states that "exceptions to the free principle apply to … dissemination of data produced by partners." Nordic lightning data comes from the shared NORDLIS network. No lightning-specific licence was found on any page, so whether this exception covers lightning is **unverified**. Safest path: don't commit raw UALF files to the repo. Ship a fetch command plus derived results only, or ask MET (frost@met.no) before redistributing the raw data.
- Basemap tiles (Esri topo) need their own attribution in the viewer.
