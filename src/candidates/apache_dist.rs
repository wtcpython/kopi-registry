use anyhow::{Context, Result};
use reqwest::Client;

use super::versioning;

pub async fn fetch_version_dirs(client: &Client, url: &str, prefix: &str) -> Result<Vec<String>> {
    let values = fetch_prefixed_dirs(client, url, prefix).await?;
    let versions = values
        .into_iter()
        .filter(|version| versioning::is_version_like(version))
        .collect();

    Ok(versioning::sort_dedup(versions))
}

pub async fn fetch_prefixed_dirs(client: &Client, url: &str, prefix: &str) -> Result<Vec<String>> {
    let body = super::get_text(client, url)
        .await
        .with_context(|| format!("fetch Apache dist listing from {url}"))?;

    Ok(hrefs(&body)
        .filter(|href| href.ends_with('/'))
        .filter_map(|href| {
            let value = href.trim_end_matches('/');
            value.strip_prefix(prefix).map(ToString::to_string)
        })
        .collect())
}

fn hrefs(body: &str) -> impl Iterator<Item = &str> {
    body.match_indices("href=\"").filter_map(|(start, _)| {
        let value_start = start + "href=\"".len();
        let value_end = body[value_start..].find('"')?;
        Some(&body[value_start..value_start + value_end])
    })
}
