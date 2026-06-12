use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{CandidateSource, MirrorMap, VersionEntry, mirrors};

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

    fn mirrors(&self) -> MirrorMap {
        mirrors(&[(
            "official",
            "https://github.com/JetBrains/kotlin/releases/download",
        )])
    }

    fn minimum_versions(&self) -> usize {
        20
    }

    async fn fetch_versions(&self, _previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let body = super::get_text_with(&client, KOTLIN_API, |request| {
            let request = request.header("User-Agent", UA);
            match github_token() {
                Some(token) => request.bearer_auth(token),
                None => request,
            }
        })
        .await
        .context("fetch Kotlin releases")?;
        let releases: Vec<GitHubRelease> =
            serde_json::from_str(&body).context("parse Kotlin JSON")?;

        let mut entries = Vec::with_capacity(releases.len());
        for release in releases {
            let ver = release.tag_name.trim_start_matches('v').to_string();
            let path = format!("/v{ver}/kotlin-compiler-{ver}.zip");
            entries.push(VersionEntry {
                version: ver,
                path,
                sha: None,
            });
        }

        let versions: Vec<String> = entries.iter().map(|entry| entry.version.clone()).collect();
        let versions = super::versioning::sort_dedup(versions);
        Ok(versions
            .iter()
            .filter_map(|version| {
                entries
                    .iter()
                    .find(|entry| &entry.version == version)
                    .cloned()
            })
            .collect())
    }
}

fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty())
}
