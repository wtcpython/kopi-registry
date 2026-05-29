# kopi-registry

Automated registry data for [kopi](https://github.com/wtc/kopi) — a JVM toolchain manager written in Rust.

The registry provides pre-computed version lists, download URLs, and checksums for all supported candidates. The kopi CLI fetches this data via [jsDelivr CDN](https://cdn.jsdelivr.net/gh/wtc/kopi-registry@main/) with a 24-hour local cache.

## Candidates

| Candidate | Source | Method |
|-----------|--------|--------|
| Maven | Maven Central | `maven-metadata.xml` (XML) |
| Gradle | Gradle Services | [`/versions/all`](https://services.gradle.org/versions/all) (JSON) |
| Kotlin | GitHub | [Releases API](https://api.github.com/repos/JetBrains/kotlin/releases) (JSON) |
| Tomcat | Apache Archive | Directory listing (HTML) |
| JMeter | Apache Archive | Directory listing (HTML) |
| Hadoop | Apache Archive | Directory listing (HTML) |

## How it works

1. A [GitHub Action](.github/workflows/generate.yml) runs daily (and on push to `src/`).
2. `cargo run --release` fetches data from each upstream source.
3. Generated JSON files are committed back to the repo — `index.json` + `candidates/*.json`.
4. The kopi CLI reads these files from jsDelivr CDN.

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
      "sha256": "abc123..."
    }
  ]
}
```

The download URL is assembled as `mirrors[mirror] + version.path`.

## License

MIT
