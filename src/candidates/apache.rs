use reqwest::Client;

use super::{MirrorMap, ShaInfo, VersionEntry, fetch_sha, mirrors};

const RECENT_CHECKSUM_WINDOW: usize = 12;

pub fn apache_mirrors(path_prefix: &str) -> MirrorMap {
    let official = format!("https://archive.apache.org/dist{path_prefix}");
    let huawei = format!("https://repo.huaweicloud.com/apache{path_prefix}");
    let tencent = format!("https://mirrors.cloud.tencent.com/apache{path_prefix}");
    let tuna = format!("https://mirrors.tuna.tsinghua.edu.cn/apache{path_prefix}");
    let ustc = format!("https://mirrors.ustc.edu.cn/apache{path_prefix}");

    mirrors(&[
        ("official", &official),
        ("huawei", &huawei),
        ("tencent", &tencent),
        ("tuna", &tuna),
        ("ustc", &ustc),
    ])
}

pub async fn entries_with_checksums<I, F>(
    client: &Client,
    official_base: &str,
    versions: I,
    path_for_version: F,
    previous: &[VersionEntry],
) -> Vec<VersionEntry>
where
    I: IntoIterator<Item = String>,
    F: Fn(&str) -> String,
{
    let has_baseline = !previous.is_empty();
    let mut entries = Vec::new();

    for (index, version) in versions.into_iter().enumerate() {
        let path = path_for_version(&version);
        let previous_sha = previous
            .iter()
            .find(|entry| {
                entry.version == version && (entry.path == path || entry.path.ends_with(&path))
            })
            .and_then(|entry| entry.sha.clone());

        if let Some(sha) = previous_sha {
            entries.push(VersionEntry {
                version,
                path,
                sha: Some(sha),
            });
            continue;
        } else if has_baseline && index >= RECENT_CHECKSUM_WINDOW {
            entries.push(VersionEntry {
                version,
                path,
                sha: None,
            });
            continue;
        }

        let sha = fetch_sha(client, official_base, &path).await;
        entries.push(VersionEntry { version, path, sha });
    }

    entries
}

pub fn parse_sha(body: &str, algo: &str) -> Option<ShaInfo> {
    body.split_whitespace()
        .filter(|token| token.len() >= 64 && token.chars().all(|c| c.is_ascii_hexdigit()))
        .max_by_key(|token| token.len())
        .map(|value| ShaInfo {
            value: value.to_string(),
            sha_type: algo.to_string(),
        })
}
