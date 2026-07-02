# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。正序排列（近期版本在上）。

## [0.1.3] — 当前

### Added
- [ ] `source/git.rs`：Git tag 读取、scope 过滤、版本一致性检查（设计文档就绪）
- [ ] `GitSourceError` 类型化错误处理
- [ ] semver 排序替代字符串排序

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
