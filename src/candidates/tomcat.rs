use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::html;
use super::{CandidateSource, VersionEntry};

const TOMCAT_PARENT: &str = "https://archive.apache.org/dist/tomcat";

pub struct Tomcat;

#[async_trait]
impl CandidateSource for Tomcat {
    fn dir_name(&self) -> &'static str {
        "tomcat"
    }

    fn display(&self) -> &'static str {
        "Tomcat"
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
        let hrefs = html::fetch_hrefs(TOMCAT_PARENT).await?;

        // Discover major versions: hrefs like "tomcat-10/" → 10
        let mut majors: Vec<u32> = hrefs
            .iter()
            .filter_map(|h| {
                let trimmed = h.trim_end_matches('/');
                trimmed
                    .strip_prefix("tomcat-")?
                    .parse::<u32>()
                    .ok()
            })
            .collect();
        majors.sort_unstable();

        let mut all_entries = Vec::new();

        for major in majors {
            let major_url = format!("{TOMCAT_PARENT}/tomcat-{major}");
            let major_hrefs = html::fetch_hrefs(&major_url).await?;

            for href in &major_hrefs {
                let trimmed = href.trim_end_matches('/');
                let ver = match trimmed.strip_prefix('v') {
                    Some(v) => v,
                    None => continue,
                };
                if !html::is_version_like(ver) {
                    continue;
                }
                let path = format!("/tomcat/tomcat-{major}/v{ver}/bin/apache-tomcat-{ver}.tar.gz");
                all_entries.push((ver.to_string(), path));
            }
        }

        // Sort and dedup
        let mut versions: Vec<String> = all_entries.iter().map(|(v, _)| v.clone()).collect();
        html::sort_versions(&mut versions);
        versions.dedup();

        let official_base = self.mirrors().get("official").cloned().unwrap_or_default();
        let client = reqwest::Client::new();

        let mut sorted = Vec::with_capacity(versions.len());
        for ver in &versions {
            if let Some((_, path)) = all_entries.iter().find(|(v, _)| v == ver) {
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
