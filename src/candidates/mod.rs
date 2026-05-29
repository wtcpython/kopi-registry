use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;

pub mod gradle;
pub mod hadoop;
pub mod html;
pub mod jmeter;
pub mod kotlin;
pub mod maven;
pub mod tomcat;

/// SHA checksum with its algorithm type.
#[derive(Debug, Clone, Serialize)]
pub struct ShaInfo {
    /// The hex-encoded hash value.
    #[serde(rename = "sha")]
    pub value: String,
    /// Algorithm: `"sha1"`, `"sha256"`, `"sha512"`, etc.
    #[serde(rename = "type")]
    pub sha_type: String,
}

/// A version entry emitted into the candidate JSON.
#[derive(Debug, Clone, Serialize)]
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
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.text().await {
                    // SHA files have two formats:
                    //   Maven-style:  "hash filename"
                    //   Hadoop-style: "SHA512 (file) = hash"
                    // Find the longest hex token (the actual hash).
                    let value = body
                        .split_whitespace()
                        .filter(|t| t.len() >= 64 && t.chars().all(|c| c.is_ascii_hexdigit()))
                        .max_by_key(|t| t.len())?
                        .to_string();
                    if !value.is_empty() {
                        return Some(ShaInfo {
                            value,
                            sha_type: algo.to_string(),
                        });
                    }
                }
            }
            _ => continue,
        }
    }

    None
}

/// Each candidate source implements this trait.
#[async_trait]
pub trait CandidateSource: Send + Sync {
    /// Directory / identifier, e.g. `"maven"`, `"gradle"`.
    fn dir_name(&self) -> &'static str;

    /// Human-readable display name, e.g. `"Maven"`, `"Gradle"`.
    fn display(&self) -> &'static str;

    /// Mirror name → base URL. `"official"` MUST always be present.
    fn mirrors(&self) -> HashMap<String, String>;

    /// Fetch all versions from upstream. Sorted newest-first (stable before pre-release).
    async fn fetch_versions(&self) -> Result<Vec<VersionEntry>>;
}
