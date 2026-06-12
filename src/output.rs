use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::candidates::{CandidateSource, MirrorMap, VersionEntry};

const CANDIDATES_DIR: &str = "candidates";

/// Final JSON shape written to `candidates/{name}.json`.
#[derive(Deserialize, Serialize)]
struct CandidateOutput {
    candidate: String,
    mirrors: MirrorMap,
    versions: Vec<VersionEntry>,
}

pub fn read_candidate_versions(root: &Path, dir_name: &str) -> Result<Vec<VersionEntry>> {
    let path = root.join(CANDIDATES_DIR).join(format!("{dir_name}.json"));
    let Ok(json) = fs::read_to_string(path) else {
        return Ok(Vec::new());
    };

    let output: CandidateOutput =
        serde_json::from_str(&json).context("parse existing candidate json")?;
    Ok(output.versions)
}

/// Entry in the top-level `index.json`.
#[derive(Serialize)]
pub struct IndexEntry {
    name: String,
    latest: String,
    detail: String,
}

pub struct CandidateWrite {
    pub entry: IndexEntry,
    pub changed: bool,
}

#[derive(Serialize)]
struct IndexOutput {
    version: u32,
    updated: String,
    candidates: BTreeMap<String, IndexEntry>,
}

/// Write `candidates/{dir_name}.json` and collect index metadata.
pub fn write_candidate(
    root: &Path,
    source: &dyn CandidateSource,
    versions: Vec<VersionEntry>,
) -> Result<CandidateWrite> {
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
    let changed = write_if_changed(&path, &json).context("write candidate json")?;

    println!(
        "  {} {}/{} ({} versions)",
        if changed { "✓" } else { "=" },
        dir,
        filename,
        output.versions.len()
    );

    Ok(CandidateWrite {
        entry: IndexEntry {
            name: source.display().to_string(),
            latest,
            detail: format!("{}/{}", dir, filename),
        },
        changed,
    })
}

/// Write the top-level `index.json`.
pub fn write_index(
    root: &Path,
    entries: Vec<(String, IndexEntry)>,
    candidate_changed: bool,
) -> Result<()> {
    let candidates: BTreeMap<String, IndexEntry> = entries.into_iter().collect();
    let path = root.join("index.json");

    let index = IndexOutput {
        version: 1,
        updated: updated_timestamp(&path, candidate_changed),
        candidates,
    };

    let json = serde_json::to_string_pretty(&index).context("serialize index")?;
    let changed = write_if_changed(&path, &json).context("write index.json")?;

    println!("  {} index.json", if changed { "✓" } else { "=" });
    Ok(())
}

fn updated_timestamp(path: &Path, data_changed: bool) -> String {
    if !data_changed
        && let Ok(existing) = fs::read_to_string(path)
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&existing)
        && let Some(updated) = value.get("updated").and_then(|v| v.as_str())
    {
        return updated.to_string();
    }

    chrono::Utc::now().to_rfc3339()
}

fn write_if_changed(path: &Path, json: &str) -> Result<bool> {
    let changed = match fs::read_to_string(path) {
        Ok(existing) => json_changed(&existing, json),
        Err(_) => true,
    };

    if changed {
        fs::write(path, json)?;
    }

    Ok(changed)
}

fn json_changed(existing: &str, generated: &str) -> bool {
    match (
        serde_json::from_str::<serde_json::Value>(existing),
        serde_json::from_str::<serde_json::Value>(generated),
    ) {
        (Ok(old), Ok(new)) => old != new,
        _ => existing != generated,
    }
}
