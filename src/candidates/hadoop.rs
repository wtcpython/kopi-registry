use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::html;
use super::{CandidateSource, VersionEntry};

const HADOOP_URL: &str = "https://archive.apache.org/dist/hadoop/common";

pub struct Hadoop;

#[async_trait]
impl CandidateSource for Hadoop {
    fn dir_name(&self) -> &'static str {
        "hadoop"
    }

    fn display(&self) -> &'static str {
        "Hadoop"
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
        let hrefs = html::fetch_hrefs(HADOOP_URL).await?;

        let mut entries = Vec::new();
        for href in &hrefs {
            let trimmed = href.trim_end_matches('/');
            let ver = match trimmed.strip_prefix("hadoop-") {
                Some(v) => v,
                None => continue,
            };
            if !html::is_version_like(ver) {
                continue;
            }
            let path = format!("/hadoop/common/hadoop-{ver}/hadoop-{ver}.tar.gz");
            entries.push((ver.to_string(), path));
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
