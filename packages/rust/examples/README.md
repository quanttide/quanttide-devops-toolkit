# examples — quanttide-devops-toolkit 使用示例

每个示例对应一个模块，展示该模块的典型用法。

## contract

### `contract`

加载契约时可能没有 contract.yaml 文件？`load_or_default` 兜底自动扫描。从 YAML 解析、
从零构建、auto_detect 三种来源覆盖了所有入口。validate 验算目录，便捷访问器按 scope
查询发布配置和测试阈值。

```sh
cargo run --example contract /path/to/repo
```

### `contract_version`

发版前手动检查版本号格式、tag 和配置文件的一致性？不——`validate_version` 帮你过，
`check_version_consistency` 帮你查，`verify_version` 一次扫描完。

```sh
cargo run --example contract_version /path/to/repo
```

## source

### `source_roadmap`

ROADMAP.md 的格式规范容易手写出错（大小写、checkbox、版本号格式）。`Roadmap::from_str`
容错解析，`validate` 逐行报告问题位置，`percent` 算出进度。扫描实际文件用 `from_path`。

```sh
cargo run --example source_roadmap
```

### `source_git_tag`

git tag 可能是 `cli/v0.1.0`、`v1.0.0` 甚至 `not-a-version`。`filter_latest_tag` /
`parse_semver_tag` 帮你安全提取最新版本。`TagSource` trait 让核心逻辑脱离 git 仓库可测，
`_with` 函数族接收任何实现了 `TagSource` 的来源。

```sh
cargo run --example source_git_tag
```

### `source_config_file`

monorepo 里多种语言共存时，`detect_languages` 独立检测每种语言不丢信息。
`read_config_versions` 统一提取 Cargo.toml / pyproject.toml / package.json / pubspec.yaml
的版本号，调用方不用关心文件格式差异。

```sh
cargo run --example source_config_file /path/to/repo
```

### `source_changelog`

读取已有 CHANGELOG 的 release notes、检查版本是否存在、追加新版本、从 git log 生成草稿。
`append_entry` 自动去重、标准化版本号、插入到最新版本之前。

```sh
cargo run --example source_changelog /path/to/repo
```

## stage

### `stage_release`

发布状态不是单一的"有没有 tag"。综合 tag 版本、配置文件版本、pending commits 三个
事实源才能判断（Unreleased / Latest / Pending / Inconsistent）。`ReleaseState` +
`Display` 把计算和输出分离。

```sh
cargo run --example stage_release
```
