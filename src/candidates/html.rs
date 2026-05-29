//! Shared helpers for HTML-scraping candidates (Tomcat, JMeter, Hadoop).

use anyhow::{Context, Result};
use reqwest::Client;

/// Fetch the HTML at `url` and extract all `href` attribute values from `<a>` tags.
pub async fn fetch_hrefs(url: &str) -> Result<Vec<String>> {
    let client = Client::new();
    let html = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("fetch {url}"))?
        .text()
        .await
        .context("read HTML")?;
    Ok(parse_hrefs(&html))
}

/// Parse `<a href="…">` values from HTML.
fn parse_hrefs(html: &str) -> Vec<String> {
    let document = scraper::Html::parse_document(html);
    let selector = match scraper::Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
        .collect()
}

/// Check whether a string looks like a version: starts with a digit and contains a dot.
pub fn is_version_like(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_digit() {
        return false;
    }
    s.contains('.')
        && s
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
}

/// Sort version strings: numeric descending, stable > pre-release.
pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| sort_key(b).cmp(&sort_key(a)));
}

fn sort_key(v: &str) -> (i32, Vec<u32>) {
    let (base, pre) = match v.split_once(['-', '+']) {
        Some((b, _)) => (b, 0i32),   // pre-release = 0
        None => (v, 1i32),            // stable = 1 (sorts first)
    };
    let parts: Vec<u32> = base.split('.').filter_map(|s| s.parse().ok()).collect();
    (pre, parts)  // stable-first: (1, ...) > (0, ...)
}
