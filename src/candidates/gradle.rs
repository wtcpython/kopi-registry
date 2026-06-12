use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{CandidateSource, MirrorMap, ShaInfo, VersionEntry, mirrors};

const GRADLE_API: &str = "https://services.gradle.org/versions/all";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GradleVersion {
    version: String,
    download_url: String,
    #[serde(default)]
    checksum: Option<String>,
}

pub struct Gradle;

#[async_trait]
impl CandidateSource for Gradle {
    fn dir_name(&self) -> &'static str {
        "gradle"
    }

    fn display(&self) -> &'static str {
        "Gradle"
    }

    fn mirrors(&self) -> MirrorMap {
        mirrors(&[
            ("official", "https://services.gradle.org/distributions"),
            ("huawei", "https://repo.huaweicloud.com/gradle"),
            ("tencent", "https://mirrors.cloud.tencent.com/gradle"),
        ])
    }

    fn minimum_versions(&self) -> usize {
        100
    }

    async fn fetch_versions(&self, _previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let body = super::get_text(&client, GRADLE_API)
            .await
            .context("fetch Gradle versions API")?;
        let all: Vec<GradleVersion> = serde_json::from_str(&body).context("parse Gradle JSON")?;

        let entries: Vec<VersionEntry> = all
            .into_iter()
            .map(|gv| {
                let path = gv
                    .download_url
                    .rsplit_once('/')
                    .map(|(_, file)| format!("/{file}"))
                    .unwrap_or_else(|| format!("/gradle-{}-bin.zip", gv.version));
                let sha = gv.checksum.map(|value| ShaInfo {
                    value,
                    sha_type: "sha256".into(),
                });
                VersionEntry {
                    version: gv.version,
                    path,
                    sha,
                }
            })
            .collect();

        let versions: Vec<String> = entries.iter().map(|e| e.version.clone()).collect();
        let versions = super::versioning::sort_dedup(versions);
        let mut sorted = Vec::with_capacity(entries.len());
        for ver in &versions {
            if let Some(e) = entries.iter().find(|e| &e.version == ver) {
                sorted.push(e.clone());
            }
        }

        Ok(sorted)
    }
}
