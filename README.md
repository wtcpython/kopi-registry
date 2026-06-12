# kopi-registry

Automated registry data for [kopi](https://github.com/wtcpython/kopi) — a JVM toolchain manager written in Rust.

The registry provides pre-computed version lists, download URLs, and checksums for all supported candidates. The kopi CLI fetches this data via [jsDelivr CDN](https://cdn.jsdelivr.net/gh/wtcpython/kopi-registry@main/) with a 24-hour local cache.

## Candidates

| Candidate | Source | Method |
|-----------|--------|--------|
| Gradle | Gradle Services | [`/versions/all`](https://services.gradle.org/versions/all) (JSON) |
| Hadoop | Apache Archive | Distribution directory listing |
| JMeter | Maven Central | `maven-metadata.xml` (XML) |
| Kotlin | GitHub | [Releases API](https://api.github.com/repos/JetBrains/kotlin/releases) (JSON) |
| Maven | Maven Central | `maven-metadata.xml` (XML) |
| Tomcat | Apache Archive | Distribution directory listing |

## How it works

1. A [GitHub Action](.github/workflows/generate.yml) runs daily or manually.
2. `cargo run --release` fetches data from each upstream source.
3. Generated JSON files are written deterministically and committed back only when data changes — `index.json` + `candidates/*.json`.
4. The kopi CLI reads these files from jsDelivr CDN.

Existing registry JSON is used as a baseline: checksums are reused for old versions, while newly discovered and recent versions are refreshed. If an upstream source has a transient network failure and existing data is available, the generator keeps the previous data instead of writing a partial registry.

CI runs formatting, clippy, tests, and a generation check to keep the committed JSON in sync with the generator.

## JSON format

### `index.json`

```json
{
  "version": 1,
  "updated": "2026-05-29T12:00:00Z",
  "candidates": {
    "maven": {
      "name": "Maven",
      "latest": "3.9.10",
      "detail": "candidates/maven.json"
    }
  }
}
```

### `candidates/maven.json`

```json
{
  "candidate": "maven",
  "mirrors": {
    "official": "https://archive.apache.org/dist",
    "huawei": "https://repo.huaweicloud.com/apache"
  },
  "versions": [
    {
      "version": "3.9.10",
      "path": "/maven/maven-3/3.9.10/binaries/apache-maven-3.9.10-bin.tar.gz",
      "sha": {
        "sha": "abc123...",
        "type": "sha512"
      }
    }
  ]
}
```

The download URL is assembled as `mirrors[mirror] + version.path`.

## License

MIT
