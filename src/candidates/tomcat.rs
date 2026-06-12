use anyhow::Result;
use async_trait::async_trait;

use super::{CandidateSource, MirrorMap, VersionEntry, apache, apache_dist, versioning};

const TOMCAT_DIST: &str = "https://archive.apache.org/dist/tomcat";
const TOMCAT_PATH_PREFIX: &str = "/tomcat";

pub struct Tomcat;

#[async_trait]
impl CandidateSource for Tomcat {
    fn dir_name(&self) -> &'static str {
        "tomcat"
    }

    fn display(&self) -> &'static str {
        "Tomcat"
    }

    fn mirrors(&self) -> MirrorMap {
        apache::apache_mirrors(TOMCAT_PATH_PREFIX)
    }

    fn minimum_versions(&self) -> usize {
        300
    }

    async fn fetch_versions(&self, previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let mut versions = Vec::new();
        let mut majors: Vec<u32> =
            apache_dist::fetch_prefixed_dirs(&client, TOMCAT_DIST, "tomcat-")
                .await?
                .into_iter()
                .filter_map(|major| major.parse().ok())
                .collect();
        majors.sort_unstable();

        for major in majors {
            let url = format!("{TOMCAT_DIST}/tomcat-{major}");
            versions.extend(apache_dist::fetch_version_dirs(&client, &url, "v").await?);
        }
        let versions = versioning::sort_dedup(versions);
        let mirrors = self.mirrors();
        let official_base = mirrors.get("official").map(String::as_str).unwrap_or("");

        Ok(apache::entries_with_checksums(
            &client,
            official_base,
            versions,
            |version| {
                let major = version.split('.').next().unwrap_or("10");
                format!("/tomcat-{major}/v{version}/bin/apache-tomcat-{version}.tar.gz")
            },
            previous,
        )
        .await)
    }
}
