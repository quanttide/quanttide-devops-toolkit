# ROADMAP — quanttide-devops

契约驱动的 DevOps 治理工具库（Rust）。

## v0.1.1（当前）

- [x] `Language::as_str()` / `BuildTool::as_str()` 显示方法
- [ ] 版本提取函数（从 CLI 移植 `read_all_config_versions`）
- [ ] 版本号校验 `validate_version()`
- [ ] `Registry` 支持 `Display`
- [ ] 文档测试覆盖
- [ ] `SourceType` 自动检测逻辑（按目录文件推断版本源类型）
- [ ] 契约验算（contract validation）：检查 scope 路径是否存在、字段是否合法
- [ ] 可选的 `clap` feature（`#[cfg(feature = "clap")]` 派生 `ValueEnum`）

## v0.1.0（已发布）

- [x] 四维契约模型：`Stage` / `Platform` / `Source` / `Scope`
- [x] YAML 序列化（`serde` 直接标注，零镜像类型）
- [x] 枚举类型替代字符串：`SourceControl`、`Pipeline`、`Registry`、`Language`、`BuildTool`
- [x] `Contract::default()` 全维度默认值
- [x] 便捷访问器：`scope_release()`、`scope_test_threshold()`、`find_scope_by_path()`
- [x] `detect_language_by_files()` 语言探测
- [x] `ContractError` 类型化错误处理
