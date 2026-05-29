use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::html;
use super::{CandidateSource, VersionEntry};

const JMETER_URL: &str = "https://archive.apache.org/dist/jmeter/binaries";

pub struct Jmeter;

#[async_trait]
impl CandidateSource for Jmeter {
    fn dir_name(&self) -> &'static str {
        "jmeter"
    }

    fn display(&self) -> &'static str {
        "JMeter"
    }

    fn mirrors(&self) -> HashMap<String, String> {
        HashMap::from([
            ("official".into(), "https://archive.apache.org/dist".into()),
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
        let hrefs = html::fetch_hrefs(JMETER_URL).await?;

        let mut entries = Vec::new();
        for href in &hrefs {
            let rest = match href.strip_prefix("apache-jmeter-") {
                Some(r) => r,
                None => continue,
            };
            let ver = rest
                .trim_end_matches(".zip")
                .trim_end_matches(".tgz")
                .trim_end_matches(".sha512")
                .trim_end_matches(".sha256")
                .trim_end_matches(".asc")
                .to_string();
            if !html::is_version_like(&ver) {
                continue;
            }
            if !rest.ends_with(".zip") && !rest.ends_with(".tgz") {
                continue;
            }
            let ext = if rest.ends_with(".tgz") { "tgz" } else { "zip" };
            let path = format!("/jmeter/binaries/apache-jmeter-{ver}.{ext}");
            entries.push((ver, path));
        }

        let mut versions: Vec<String> = entries.iter().map(|(v, _)| v.clone()).collect();
        html::sort_versions(&mut versions);
        versions.dedup();

        let official_base = self.mirrors().get("official").cloned().unwrap_or_default();
        let client = reqwest::Client::new();

        let mut sorted = Vec::with_capacity(versions.len());
        for ver in &versions {
            if let Some((_, path)) = entries.iter().find(|(v, _)| v == ver) {
                let sha = super::fetch_sha(&client, &official_base, path).await;
                sorted.push(VersionEntry {
                    version: ver.clone(),
                    path: path.clone(),
                    sha,
                });
            }
        }

        Ok(sorted)
    }
}
