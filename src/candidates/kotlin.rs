use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::{CandidateSource, VersionEntry};

const KOTLIN_API: &str = "https://api.github.com/repos/JetBrains/kotlin/releases?per_page=100";
const UA: &str = concat!("kopi-registry/", env!("CARGO_PKG_VERSION"));

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub struct Kotlin;

#[async_trait]
impl CandidateSource for Kotlin {
    fn dir_name(&self) -> &'static str {
        "kotlin"
    }

    fn display(&self) -> &'static str {
        "Kotlin"
    }

    fn mirrors(&self) -> HashMap<String, String> {
        HashMap::from([(
            "official".into(),
            "https://github.com/JetBrains/kotlin/releases/download".into(),
        )])
    }

    async fn fetch_versions(&self) -> Result<Vec<VersionEntry>> {
        let client = Client::new();
        let releases: Vec<GitHubRelease> = client
            .get(KOTLIN_API)
            .header("User-Agent", UA)
            .send()
            .await
            .context("fetch Kotlin releases")?
            .json()
            .await
            .context("parse Kotlin JSON")?;

        let official_base = self.mirrors().get("official").cloned().unwrap_or_default();

        let mut entries = Vec::with_capacity(releases.len());
        for r in releases {
            let ver = r.tag_name.trim_start_matches('v').to_string();
            let path = format!("/v{ver}/kotlin-compiler-{ver}.zip");
            let sha = super::fetch_sha(&client, &official_base, &path).await;
            entries.push(VersionEntry { version: ver, path, sha });
        }

        // Sort stable-first
        let mut versions: Vec<String> = entries.iter().map(|e| e.version.clone()).collect();
        super::html::sort_versions(&mut versions);
        versions.dedup();
        let mut sorted = Vec::with_capacity(entries.len());
        for ver in &versions {
            if let Some(e) = entries.iter().find(|e| &e.version == ver) {
                sorted.push(e.clone());
            }
        }

        Ok(sorted)
    }
}
