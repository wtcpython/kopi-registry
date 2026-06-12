# kopi-registry

[kopi](https://github.com/wtcpython/kopi) 的自动化注册表数据源。kopi CLI 通过 [jsDelivr CDN](https://cdn.jsdelivr.net/gh/wtcpython/kopi-registry@main/) 获取预计算的版本列表、下载链接和校验码，本地缓存 24 小时。

## 候选软件

| Candidate | 数据源 | 获取方式 |
|-----------|--------|----------|
| Gradle | Gradle Services | [`/versions/all`](https://services.gradle.org/versions/all) (JSON) |
| Hadoop | Apache Archive | 发行包目录列表 |
| JMeter | Maven Central | `maven-metadata.xml` (XML) |
| Kotlin | GitHub | [Releases API](https://api.github.com/repos/JetBrains/kotlin/releases) (JSON) |
| Maven | Maven Central | `maven-metadata.xml` (XML) |
| Tomcat | Apache Archive | 发行包目录列表 |

## 工作流程

1. [GitHub Action](.github/workflows/generate.yml) 每天自动运行，也可以手动触发。
2. `cargo run --release` 从各上游获取最新数据。
3. 生成器会稳定输出 JSON，只有数据实际变化时才提交回仓库：`index.json` + `candidates/*.json`。
4. kopi CLI 通过 jsDelivr CDN 读取这些文件。

生成器会把已有 registry JSON 作为基线：旧版本复用已有 checksum，只刷新新增版本和最近版本。如果上游临时网络失败且本地已有数据，生成器会沿用旧数据，而不是写出半残 registry。

CI 会运行格式检查、clippy、测试，以及生成结果校验，确保仓库中的 JSON 和生成器保持一致。

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
      "sha": {
        "sha": "abc123...",
        "type": "sha512"
      }
    }
  ]
}
```

下载链接由客户端拼接：`mirrors[镜像名] + version.path`。

## License

MIT
