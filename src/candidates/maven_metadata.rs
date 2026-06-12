use anyhow::{Context, Result};
use reqwest::Client;

use super::versioning;

pub async fn fetch_versions(client: &Client, url: &str) -> Result<Vec<String>> {
    let xml = super::get_text(client, url)
        .await
        .with_context(|| format!("fetch Maven metadata from {url}"))?;

    Ok(versioning::sort_dedup(parse_versions(&xml)))
}

fn parse_versions(xml: &str) -> Vec<String> {
    let Some(start) = xml.find("<versions>") else {
        return Vec::new();
    };
    let start = start + "<versions>".len();
    let Some(end) = xml[start..].find("</versions>") else {
        return Vec::new();
    };

    let mut versions = Vec::new();
    let mut rest = &xml[start..start + end];

    while let Some(tag_start) = rest.find("<version>") {
        let content_start = tag_start + "<version>".len();
        rest = &rest[content_start..];
        let Some(tag_end) = rest.find("</version>") else {
            break;
        };

        let version = rest[..tag_end].trim();
        if !version.is_empty() {
            versions.push(version.to_string());
        }
        rest = &rest[tag_end + "</version>".len()..];
    }

    versions
}
