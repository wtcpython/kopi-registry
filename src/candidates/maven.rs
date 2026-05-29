use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;

use super::{CandidateSource, VersionEntry};

const MAVEN_METADATA: &str =
    "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml";

pub struct Maven;

#[async_trait]
impl CandidateSource for Maven {
    fn dir_name(&self) -> &'static str {
        "maven"
    }

    fn display(&self) -> &'static str {
        "Maven"
    }

    fn mirrors(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "official".into(),
                "https://archive.apache.org/dist".into(),
            ),
            (
                "huawei".into(),
                "https://repo.huaweicloud.com/apache".into(),
            ),
            (
                "tencent".into(),
                "https://mirrors.cloud.tencent.com/apache".into(),
            ),
            (
                "tuna".into(),
                "https://mirrors.tuna.tsinghua.edu.cn/apache".into(),
            ),
            (
                "ustc".into(),
                "https://mirrors.ustc.edu.cn/apache".into(),
            ),
        ])
    }

    async fn fetch_versions(&self) -> Result<Vec<VersionEntry>> {
        let client = Client::new();
        let xml = client
            .get(MAVEN_METADATA)
            .send()
            .await
            .context("fetch maven-metadata.xml")?
            .text()
            .await
            .context("read maven-metadata.xml")?;

        let versions = parse_maven_metadata(&xml);
        let official_base = self.mirrors().get("official").cloned().unwrap_or_default();

        let mut entries = Vec::with_capacity(versions.len());
        for ver in versions {
            let major = ver.split('.').next().unwrap_or("3");
            let path =
                format!("/maven/maven-{major}/{ver}/binaries/apache-maven-{ver}-bin.tar.gz");
            let sha = super::fetch_sha(&client, &official_base, &path).await;
            entries.push(VersionEntry {
                version: ver,
                path,
                sha,
            });
        }

        Ok(entries)
    }
}

/// Parse `<version>…</version>` entries from maven-metadata.xml
/// without a full XML parser — the schema is simple and stable.
fn parse_maven_metadata(xml: &str) -> Vec<String> {
    let mut versions = Vec::new();

    let start = match xml.find("<versions>") {
        Some(p) => p + "<versions>".len(),
        None => return versions,
    };
    let end = match xml[start..].find("</versions>") {
        Some(p) => start + p,
        None => return versions,
    };
    let block = &xml[start..end];

    let mut rest = block;
    while let Some(tag_start) = rest.find("<version>") {
        let content_start = tag_start + "<version>".len();
        rest = &rest[content_start..];
        if let Some(tag_end) = rest.find("</version>") {
            let ver = rest[..tag_end].trim().to_string();
            rest = &rest[tag_end + "</version>".len()..];
            if !ver.is_empty() {
                versions.push(ver);
            }
        } else {
            break;
        }
    }

    super::html::sort_versions(&mut versions);
    versions.dedup();
    versions
}
