use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::candidates::{CandidateSource, VersionEntry};

const CANDIDATES_DIR: &str = "candidates";

/// Final JSON shape written to `candidates/{name}.json`.
#[derive(Serialize)]
struct CandidateOutput {
    candidate: String,
    mirrors: HashMap<String, String>,
    versions: Vec<VersionEntry>,
}

/// Entry in the top-level `index.json`.
#[derive(Serialize)]
pub struct IndexEntry {
    name: String,
    latest: String,
    detail: String,
}

#[derive(Serialize)]
struct IndexOutput {
    version: u32,
    updated: String,
    candidates: HashMap<String, IndexEntry>,
}

/// Write `candidates/{dir_name}.json` and collect index metadata.
pub fn write_candidate(root: &PathBuf, source: &dyn CandidateSource, versions: Vec<VersionEntry>) -> Result<IndexEntry> {
    let dir = CANDIDATES_DIR;
    fs::create_dir_all(root.join(dir)).context("create candidates dir")?;

    let latest = versions
        .first()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "unknown".into());

    let output = CandidateOutput {
        candidate: source.dir_name().to_string(),
        mirrors: source.mirrors(),
        versions,
    };

    let filename = format!("{}.json", source.dir_name());
    let path = root.join(dir).join(&filename);
    let json = serde_json::to_string_pretty(&output).context("serialize candidate json")?;
    fs::write(&path, json).context("write candidate json")?;

    println!("  ✓ {}/{} ({} versions)", dir, filename, output.versions.len());

    Ok(IndexEntry {
        name: source.display().to_string(),
        latest,
        detail: format!("{}/{}", dir, filename),
    })
}

/// Write the top-level `index.json`.
pub fn write_index(root: &PathBuf, entries: Vec<(String, IndexEntry)>) -> Result<()> {
    let candidates: HashMap<String, IndexEntry> = entries.into_iter().collect();

    let index = IndexOutput {
        version: 1,
        updated: chrono::Utc::now().to_rfc3339(),
        candidates,
    };

    let path = root.join("index.json");
    let json = serde_json::to_string_pretty(&index).context("serialize index")?;
    fs::write(&path, json).context("write index.json")?;

    println!("  ✓ index.json");
    Ok(())
}
