# kopi-registry

[kopi](https://github.com/wtc/kopi) 的自动化注册表数据源。kopi CLI 通过 [jsDelivr CDN](https://cdn.jsdelivr.net/gh/wtc/kopi-registry@main/) 获取预计算的版本列表、下载链接和校验码，本地缓存 24 小时。

## 候选软件

| Candidate | 数据源 | 获取方式 |
|-----------|--------|----------|
| Maven | Maven Central | `maven-metadata.xml` (XML) |
| Gradle | Gradle Services | [`/versions/all`](https://services.gradle.org/versions/all) (JSON) |
| Kotlin | GitHub | [Releases API](https://api.github.com/repos/JetBrains/kotlin/releases) (JSON) |
| Tomcat | Apache Archive | 目录列表 (HTML) |
| JMeter | Apache Archive | 目录列表 (HTML) |
| Hadoop | Apache Archive | 目录列表 (HTML) |

## 工作流程

1. [GitHub Action](.github/workflows/generate.yml) 每天自动运行（修改 `src/` 时也会触发）。
2. `cargo run --release` 从各上游获取最新数据。
3. 生成的 JSON 文件提交回仓库：`index.json` + `candidates/*.json`。
4. kopi CLI 通过 jsDelivr CDN 读取这些文件。

## JSON 格式

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

下载链接由客户端拼接：`mirrors[镜像名] + version.path`。

## License

MIT
