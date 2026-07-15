# CHANGELOG

## [0.3.1] - 2026-07-15

### Added

- `stage::release::ReleaseStatus` — 发布生命周期状态枚举（Unreleased / Latest / Pending / Inconsistent / Unknown）。
- `stage::release::ReleaseState` — 发布状态快照结构体。
- `source::git_repo::is_git_repo()` — 判断路径是否为 git 仓库，公开并添加覆盖测试。

## [0.3.0] - 2026-07-06

### Breaking

- `source::git_tag::latest_tag()` 返回原始 tag 名（如 `cli/v0.2.0`），不再返回标准化版本号。改用新增的 `latest_version()` 获取标准化版本号（如 `0.2.0`）。

### Added

- `source::git_tag::latest_version()` — 获取最新版本号（标准化，去 scope/v 前缀）。
- `source::git_tag::latest_version_with()` — 带 TagSource 注入的 `latest_version`。
- `source::git_tag::filter_latest_version()` — 纯函数，从 tag 列表中选最新版本号。
- `source::git_tag::parse_semver_tag()` 改为 `pub` — 外部可复用。
- `source::git_tag` 模块文档补充 `latest_version` 示例。

### Changed

- `Roadmap::from_str()` 首行校验从精确匹配 `# ROADMAP` 改为 `starts_with("# ROADMAP")`，支持 `# ROADMAP — cli` 等后缀。
- `source::changelog` 改用 gix 替代 CLI git，复用 `contract::normalize_version`。

## [0.2.3] - 2026-07-06

### Added
- 新增 `source::changelog` 模块，实现 `collect_git_log`、`build_changelog_prompt` 和 `append_entry` 功能。

### Changed
- 更新文档，标记 v0.2.3 开发任务完成，同步测试命名，并规划未来开发任务。
- 重构 changelog 示例，重写为自包含的 toolkit API 演示。

### Fixed
- 修复 `collect_git_log` 实现，改用 gix 库，并复用 `contract::normalize_version` 进行版本规范化。

## [0.2.2] - 2026-07-06

### Added

- `source::config_file::detect_languages(dir) -> Vec<Language>` — 独立检测目录下所有编程语言

### Changed

- `Contract::resolve_language()` / `auto_detect()` 内部改用 `detect_languages`
- `detect_language()` 标记为 `#[deprecated]`

### Fixed

- 示例和集成测试统一加 `#[allow(deprecated)]`，消除编译警告

## [0.2.1] - 2026-07-05

### Added

- `Language::default_build_tool()` — Language → BuildTool 映射
- `Language::default_registry()` — Language → Registry 映射
- `Contract::auto_detect(repo_path)` — 扫描 `src/`、`packages/`、`apps/` 自动生成契约
- `contract::load_or_default(repo_path)` — load 失败则 auto_detect
- `Contract::resolve_language()` — scope 语言声明兜底探测
- 36 个补充测试（auto_detect 集成、Language 默认方法、semver 边界、RoadmapError display）

### Changed

- 测试覆盖率提升至 99.05%（519/524）
- 示例 `contract.rs` 从 164 行缩减至 63 行，利用新 API
- CI workflow 统一命名：`publish-*` / `test-*` → `release-*`
- `release-rust.yml`：合并 check（测试+覆盖率）→ release（cargo publish）

### Fixed

- 6 个 clippy warning（`should_implement_trait`、`manual_strip`、`collapsible_if`、`loop_counter`）

### Removed

- `examples/git_submodule.rs`（git2 示例，逻辑已进入实验室阶段）

## [0.2.0] - 2026-07-05

### Added

- `VersionSource` trait（原 `TagSource`）
- `source::language` 模块
- 35 个单元测试覆盖纯逻辑全分支
- 7 个集成测试覆盖 I/O 边界
- 10 个版本号异常场景测试（pre-release、build metadata、大写 V）
- 新增 pre_publish 配置（package archetype）

### Changed

- **破坏性重构**：`source::git` → `source::git_tag` + `source::config_file`，职责分离
- **破坏性重构**：`version_status` → `verify_version`，移至 `contract::version`
- **破坏性重构**：`VersionStatus` → `VersionState`，命名反映多字段快照语义
- **破坏性重构**：`detect_language_by_files` → `source::config_file::detect_language`
- **破坏性重构**：`read_all_config_versions` → `source::config_file::read_config_versions`
- **破坏性重构**：`VersionSourceError` → `TagError`
- git tag 读取从 `git2` 迁移至 `gix`（快 14x）
- 引入 `semver` crate 替代手写 semver 解析
- `Contract::load()` 现在展开 `Auto` → `SourceType::detect()`
- 测试文件按模块一一对应拆分（contract_* / source_*）
- 所有测试零 CLI git 依赖
- 将默认测试阈值从 70% 提升至 80%
- 添加 GitSubmoduleEditor 及 submodule 示例，更新文档以反映 gix 优先的混合策略

## [0.1.5] - 2026-07-03

### Added

- `source::roadmap` 模块：ROADMAP.md 解析、进度统计、格式验证（纯文本，零新增依赖）
- `RoadmapVersion`、`RoadmapProgress`、`RoadmapChecklistItem`、`RoadmapIssue` 类型
- 101 单元测试覆盖解析、进度统计、格式验证、v 前缀标准化
- 5 集成测试覆盖端到端解析、错误显示、真实文件验证

### Fixed

- `test-rust` CI coverage 阈值检查：LCOV 直解析增加空 LF/LH 守卫和除零防护

### Coverage

- 行覆盖率：98.85%（514 / 520）
- 全部 142 测试：101 单元 + 36 集成 + 5 文档

## [0.1.4] - 2026-07-03

### Added

- `source::changelog` 模块：CHANGELOG 解析、release notes 提取、版本存在性校验（`parse-changelog`）
- 集成测试套件：15 个测试覆盖 contract 加载、SourceType 检测、Registry 序列化
- `.githooks/pre-commit`：Rust 变更自动跑测试
- `test-rust` CI workflow：release 触发，build + test + coverage 阈值 95%

### Changed

- 依赖：新增 `parse-changelog = "0.6"`（零额外依赖）
- `contract/mod.rs` 的 `load()` / `load_from_str()` 调用者以外不再直接使用
- `contract/source.rs` 的 `SourceType::detect()` 通过集成测试覆盖全部 5 种文件

### Coverage

- 单位测试覆盖率：98.39%（区域）/ 99.31%（行）
- 全部 115 测试：75 单元 + 36 集成 + 4 文档

## [0.1.3] - 2026-07-03

### Added

- `source::git` 模块：Git tag 读取、scope 过滤、版本一致性检查
- `GitSourceError` 类型化错误处理
- `VersionStatus` 版本一致性检查结果
- semver 排序（修复字符串排序 `v10 < v9` 问题）
- 43 单元测试覆盖 tag 读取、scope 过滤、semver 排序

### Changed

- 依赖：新增 `git2 = "0.19"`（必选）
- `contract/source.rs` 保持单文件，`source/` 以顶层模块存在
- `lib.rs` 新增 `pub mod source`

## [0.1.2] - 2026-07-02

### Changed

- `contract/model.rs` 拆分为 4 个独立模块：`stage.rs`、`platform.rs`、`source.rs`、`scope.rs`
- `model.rs` 更名为 `core.rs`

### Added

- `version.rs` 独立模块（已有，确认结构）
- README.md、CHANGELOG.md（首次补充）

## [0.1.1] - 2026-07-02

### Added

- `Language::as_str()` / `BuildTool::as_str()` 显示方法
- `Registry: Display`
- `SourceType::detect()` 按文件自动检测
- `Contract::validate()` 契约验算（scope 目录存在性检查）
- `validate_version()` / `normalize_version()` 版本号工具
- `read_all_config_versions()` 配置文件版本提取
- 可选的 `clap` feature（`SourceControl` / `Pipeline` / `Registry` / `SourceType`）
- `version.rs` 独立模块（31 单元测试 + 4 文档测试）

## [0.1.0] - 2026-07-02

### Added

- 四维契约模型：`Stage` / `Platform` / `Source` / `Scope`
- YAML 序列化（`serde` 直接标注，零镜像类型）
- 枚举类型替代字符串：`SourceControl`、`Pipeline`、`Registry`、`Language`、`BuildTool`
- `Contract::default()` 全维度默认值
- 便捷访问器：`scope_release()`、`scope_test_threshold()`、`find_scope_by_path()`
- `detect_language_by_files()` 语言探测
- `ContractError` 类型化错误处理
