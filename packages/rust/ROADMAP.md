# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。正序排列（近期版本在上）。

## [0.1.5] — 已实施

### Added
- [x] `source/roadmap.rs`：ROADMAP.md 解析、进度统计、格式验证
- [x] `RoadmapProgress` / `RoadmapIssue` 类型
- [x] 依赖：无新增（纯文本解析）

### Fixed
- [x] `test-rust` CI coverage 阈值检查：LCOV 直解析（`LF`/`LH`）

## [0.1.4] — 已发布

### Added
- [x] `source/changelog.rs`：CHANGELOG 解析、release notes 提取、版本存在性校验
- [x] 依赖 `parse-changelog`（taiki-e，274k 下载，只读轻量）
- [x] 集成测试套件：contract 加载、SourceType 检测、Registry 序列化（15 测试）
- [x] `.githooks/pre-commit`：Rust 变更自动跑测试
- [x] `test-rust` CI workflow：release 触发，coverage 阈值 95%

## [0.1.3] — 已发布

### Added
- [x] `source/git.rs`：Git tag 读取、scope 过滤、版本一致性检查
- [x] `GitSourceError` 类型化错误处理
- [x] semver 排序替代字符串排序（43 测试全部通过）

## [0.1.2] — 已发布

### Added
- [x] `Language::as_str()` / `BuildTool::as_str()` 显示方法
- [x] `Registry: Display`
- [x] `SourceType::detect()` 按文件自动检测
- [x] `Contract::validate()` 契约验算（scope 目录存在性检查）
- [x] `validate_version()` / `normalize_version()` 版本号工具
- [x] `read_all_config_versions()` 配置文件版本提取
- [x] 可选的 `clap` feature（`SourceControl` / `Pipeline` / `Registry` / `SourceType`）
- [x] `version.rs` 独立模块（31 单元测试 + 4 文档测试）

## [0.1.0] — 已发布

### Added
- [x] 四维契约模型：`Stage` / `Platform` / `Source` / `Scope`
- [x] YAML 序列化（`serde` 直接标注，零镜像类型）
- [x] 枚举类型替代字符串：`SourceControl`、`Pipeline`、`Registry`、`Language`、`BuildTool`
- [x] `Contract::default()` 全维度默认值
- [x] 便捷访问器：`scope_release()`、`scope_test_threshold()`、`find_scope_by_path()`
- [x] `detect_language_by_files()` 语言探测
- [x] `ContractError` 类型化错误处理
