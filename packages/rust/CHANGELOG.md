# CHANGELOG

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
