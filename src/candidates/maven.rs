use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{CandidateSource, MirrorMap, VersionEntry, maven_metadata, mirrors};

const MAVEN_METADATA: &str =
    "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml";
const MAVEN_PATH_PREFIX: &str = "/maven";

pub struct Maven;

#[async_trait]
impl CandidateSource for Maven {
    fn dir_name(&self) -> &'static str {
        "maven"
    }

    fn display(&self) -> &'static str {
        "Maven"
    }

    fn mirrors(&self) -> MirrorMap {
        mirrors(&[
            ("official", "https://archive.apache.org/dist/maven"),
            ("huawei", "https://repo.huaweicloud.com/apache/maven"),
            ("tencent", "https://mirrors.cloud.tencent.com/apache/maven"),
            ("tuna", "https://mirrors.tuna.tsinghua.edu.cn/apache/maven"),
            ("ustc", "https://mirrors.ustc.edu.cn/apache/maven"),
        ])
    }

    fn minimum_versions(&self) -> usize {
        20
    }

    async fn fetch_versions(&self, previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let versions = maven_metadata::fetch_versions(&client, MAVEN_METADATA)
            .await
            .context("fetch Maven versions")?;
        let official_base = self.mirrors().get("official").cloned().unwrap_or_default();

        let mut entries = Vec::with_capacity(versions.len());
        let has_baseline = !previous.is_empty();
        for (index, ver) in versions.into_iter().enumerate() {
            let major = ver.split('.').next().unwrap_or("3");
            let path = format!("/maven-{major}/{ver}/binaries/apache-maven-{ver}-bin.tar.gz");
            let full_path = format!("{MAVEN_PATH_PREFIX}{path}");
            let previous_sha = previous
                .iter()
                .find(|entry| {
                    entry.version == ver && (entry.path == path || entry.path == full_path)
                })
                .and_then(|entry| entry.sha.clone());
            let sha = if let Some(sha) = previous_sha {
                if index >= 12 || !sha.value.is_empty() {
                    Some(sha)
                } else {
                    super::fetch_sha(&client, &official_base, &path).await
                }
            } else if has_baseline && index >= 12 {
                None
            } else {
                super::fetch_sha(&client, &official_base, &path).await
            };
            entries.push(VersionEntry {
                version: ver,
                path,
                sha,
            });
        }

        Ok(entries)
    }
}
