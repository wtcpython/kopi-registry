use anyhow::Result;
use async_trait::async_trait;

use super::{CandidateSource, MirrorMap, VersionEntry, apache, maven_metadata};

const JMETER_METADATA: &str =
    "https://repo.maven.apache.org/maven2/org/apache/jmeter/ApacheJMeter/maven-metadata.xml";
const JMETER_PATH_PREFIX: &str = "/jmeter/binaries";

pub struct Jmeter;

#[async_trait]
impl CandidateSource for Jmeter {
    fn dir_name(&self) -> &'static str {
        "jmeter"
    }

    fn display(&self) -> &'static str {
        "JMeter"
    }

    fn mirrors(&self) -> MirrorMap {
        apache::apache_mirrors(JMETER_PATH_PREFIX)
    }

    fn minimum_versions(&self) -> usize {
        20
    }

    async fn fetch_versions(&self, previous: &[VersionEntry]) -> Result<Vec<VersionEntry>> {
        let client = super::client()?;
        let versions = maven_metadata::fetch_versions(&client, JMETER_METADATA).await?;
        let mirrors = self.mirrors();
        let official_base = mirrors.get("official").map(String::as_str).unwrap_or("");
        Ok(apache::entries_with_checksums(
            &client,
            official_base,
            versions,
            |version| format!("/apache-jmeter-{version}.tgz"),
            previous,
        )
        .await)
    }
}
