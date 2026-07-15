# examples — quanttide-devops-toolkit 使用示例

每个示例对应一个模块，展示该模块的典型用法。

## contract

### `contract`

契约加载、从 YAML 解析、便捷访问器、目录验算、版本状态检查、自动推测。不传路径时使用当前目录；传路径时加载指定仓库的契约。

```sh
cargo run --example contract /path/to/repo
```

### `version`

版本号格式校验（validate_version）、标准化（normalize_version）、tag 与配置文件一致性检查（check_version_consistency）。不传路径时只运行纯函数部分；传路径时额外执行仓库级别的 verify_version。

```sh
cargo run --example version /path/to/repo
```

## source

### `roadmap`

ROADMAP.md 解析、版本进度统计（每版本完成率）、格式验证（版本号/分类大小写/checkbox 合规性）。

```sh
cargo run --example roadmap
```

### `git_tag`

tag 过滤、semver 解析、`TagSource` trait + mock 注入：纯函数（`parse_semver_tag` / `filter_tags_by_scope` / `filter_latest_tag` / `filter_latest_version`）和 `_with` 系列函数（`latest_tag_with` / `latest_version_with` / `tags_for_scope_with`）。无需 git 仓库。

```sh
cargo run --example git_tag
```

### `config_file`

目录语言检测（detect_languages）、配置文件版本号读取（read_config_versions）。不传路径时创建临时目录演示；传路径时扫描指定目录。

```sh
cargo run --example config_file /path/to/repo
```

### `changelog`

解析与查询（`from_str` / `contains_version` / `release_notes` / `versions` / `latest_version`）、条目追加（`append_entry`）、生成 pipeline（`collect_git_log` → `build_changelog_prompt`）。不传路径时运行前两部分；传路径时额外执行生成。

```sh
cargo run --example changelog /path/to/repo
```

## stage

### `release`

结合 git tag（通过 mock `TagSource`）与契约配置，计算多 scope 的发布状态（Unreleased / Latest / Pending / Inconsistent），输出结构化报告。

```sh
cargo run --example release
```
