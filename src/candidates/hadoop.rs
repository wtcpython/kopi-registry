use anyhow::Result;
use async_trait::async_trait;

use super::{CandidateSource, MirrorMap, VersionEntry, apache, apache_dist};

const HADOOP_DIST: &str = "https://archive.apache.org/dist/hadoop/common";
const HADOOP_PATH_PREFIX: &str = "/hadoop/common";

pub struct Hadoop;

#[async_trait]
impl CandidateSource for Hadoop {
    fn dir_name(&self) -> &'static str {
        "hadoop"
    }

    fn display(&self) -> &'static str {
        "Hadoop"
    }

    fn mirrors(&self) -> MirrorMap {
        apache::apache_mirrors(HADOOP_PATH_PREFIX)
    }

    fn minimum_versions(&self) -> usize {
        100
    }

    async fn fetch_versions(&self, previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let versions = apache_dist::fetch_version_dirs(&client, HADOOP_DIST, "hadoop-").await?;
        let mirrors = self.mirrors();
        let official_base = mirrors.get("official").map(String::as_str).unwrap_or("");
        Ok(apache::entries_with_checksums(
            &client,
            official_base,
            versions,
            |version| format!("/hadoop-{version}/hadoop-{version}.tar.gz"),
            previous,
        )
        .await)
    }
}
