pub mod geojson;
pub mod gpkg;

use crate::audio::spectrogram::Spectrogram;
use crate::onset::solve::OnsetSolution;
use crate::search::pipeline::SearchResult;
use crate::search::trace::Traces;
use anyhow::Result;
use serde_json::json;
use std::path::Path;

const VIEWER_TEMPLATE: &str = include_str!("../../assets/viewer.html");

/// Writes heatmap.geojson, result.gpkg, candidates.json, trace.json and a self-contained index.html viewer into `dir`.
pub fn write_all(
    dir: &Path,
    result: &SearchResult,
    onsets: &[OnsetSolution],
    traces: &Traces,
    spec: &Spectrogram,
    run_info: serde_json::Value,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let fc = geojson::feature_collection(&result.cells, &result.regions, onsets);
    let fc_text = serde_json::to_string(&fc)?;
    std::fs::write(dir.join("heatmap.geojson"), &fc_text)?;
    gpkg::write(&dir.join("result.gpkg"), &fc)?;
    let candidates = json!({ "run": run_info, "regions": result.regions, "onset_solutions": onsets });
    std::fs::write(dir.join("candidates.json"), serde_json::to_string_pretty(&candidates)?)?;
    let (traces_text, spec_text) = (serde_json::to_string(traces)?, serde_json::to_string(spec)?);
    std::fs::write(dir.join("trace.json"), format!(r#"{{"traces":{traces_text},"spectrogram":{spec_text}}}"#))?;
    // Inlined so the viewer works from file:// without a server.
    let html = VIEWER_TEMPLATE
        .replace("/*GEOJSON*/null", &fc_text)
        .replace("/*RUN*/null", &serde_json::to_string(&candidates["run"])?)
        .replace("/*TRACES*/null", &traces_text)
        .replace("/*SPEC*/null", &spec_text);
    std::fs::write(dir.join("index.html"), html)?;
    Ok(())
}
