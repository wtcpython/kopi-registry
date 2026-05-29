use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::{CandidateSource, ShaInfo, VersionEntry};

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

    fn mirrors(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "official".into(),
                "https://services.gradle.org/distributions".into(),
            ),
            (
                "huawei".into(),
                "https://repo.huaweicloud.com/gradle".into(),
            ),
            (
                "tencent".into(),
                "https://mirrors.cloud.tencent.com/gradle".into(),
            ),
        ])
    }

    async fn fetch_versions(&self) -> Result<Vec<VersionEntry>> {
        let client = Client::new();
        let all: Vec<GradleVersion> = client
            .get(GRADLE_API)
            .send()
            .await
            .context("fetch Gradle versions API")?
            .json()
            .await
            .context("parse Gradle JSON")?;

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
