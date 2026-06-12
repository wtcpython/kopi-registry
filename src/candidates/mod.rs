use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

pub mod apache;
pub mod apache_dist;
pub mod gradle;
pub mod hadoop;
pub mod jmeter;
pub mod kotlin;
pub mod maven;
pub mod maven_metadata;
pub mod tomcat;
pub mod versioning;

pub type MirrorMap = BTreeMap<String, String>;

pub fn mirrors(entries: &[(&str, &str)]) -> MirrorMap {
    entries
        .iter()
        .map(|(name, url)| ((*name).to_string(), (*url).to_string()))
        .collect()
}

/// SHA checksum with its algorithm type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShaInfo {
    /// The hex-encoded hash value.
    #[serde(rename = "sha")]
    pub value: String,
    /// Algorithm: `"sha1"`, `"sha256"`, `"sha512"`, etc.
    #[serde(rename = "type")]
    pub sha_type: String,
}

/// A version entry emitted into the candidate JSON.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionEntry {
    pub version: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<ShaInfo>,
}

/// Fetch the SHA checksum for a version by trying known algorithms in order
/// (sha512 → sha256 → sha1). The SHA URL is `{official_base}{path}.{algo}`.
pub async fn fetch_sha(client: &Client, official_base: &str, path: &str) -> Option<ShaInfo> {
    let base = official_base.trim_end_matches('/');
    let download_url = format!("{}{}", base, path);

    for algo in ["sha512", "sha256"] {
        let url = format!("{download_url}.{algo}");
        if let Some(body) = get_optional_text(client, &url).await
            && let Some(sha) = apache::parse_sha(&body, algo)
        {
            return Some(sha);
        }
    }

    None
}

const HTTP_ATTEMPTS: usize = 3;

pub fn client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("build HTTP client")
}

pub async fn get_text(client: &Client, url: &str) -> Result<String> {
    get_text_with(client, url, |request| request).await
}

pub async fn get_text_with<F>(client: &Client, url: &str, configure: F) -> Result<String>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut last_error = None;

    for attempt in 1..=HTTP_ATTEMPTS {
        match configure(client.get(url)).send().await {
            Ok(response) if response.status().is_success() => {
                return response
                    .text()
                    .await
                    .with_context(|| format!("read response body from {url}"));
            }
            Ok(response) if response.status().is_client_error() => {
                bail!("GET {url} failed: HTTP {}", response.status());
            }
            Ok(response) => {
                last_error = Some(format!("HTTP {}", response.status()));
            }
            Err(error) => {
                last_error = Some(error.to_string());
            }
        }

        if attempt < HTTP_ATTEMPTS {
            sleep(Duration::from_millis(250 * attempt as u64)).await;
        }
    }

    bail!(
        "GET {url} failed after {HTTP_ATTEMPTS} attempts: {}",
        last_error.unwrap_or_else(|| "unknown error".to_string())
    )
}

pub async fn get_optional_text(client: &Client, url: &str) -> Option<String> {
    get_text(client, url).await.ok()
}

/// Each candidate source implements this trait.
#[async_trait]
pub trait CandidateSource: Send + Sync {
    /// Directory / identifier, e.g. `"maven"`, `"gradle"`.
    fn dir_name(&self) -> &'static str;

    /// Human-readable display name, e.g. `"Maven"`, `"Gradle"`.
    fn display(&self) -> &'static str;

    /// Mirror name → base URL. `"official"` MUST always be present.
    fn mirrors(&self) -> MirrorMap;

    /// Sanity floor used to catch stale or incomplete upstream source parsing.
    fn minimum_versions(&self) -> usize {
        1
    }

    /// Fetch all versions from upstream. Sorted newest-first (stable before pre-release).
    async fn fetch_versions(&self, previous: &[VersionEntry]) -> Result<Vec<VersionEntry>>;
}
