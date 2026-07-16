# ROADMAP — quanttide-devops-toolkit (Rust)

> 共享 SDK，提供契约、source、changelog 等基础设施。

## 技术债务

### 超长函数（code audit）

- [ ] `source/roadmap.rs` 两处超长：`fn` 85 行 + `fn` 66 行 — 提取子函数
- [ ] `source/changelog.rs` 两处超长：`fn` 49 行 + `fn` 48 行 — 提取子函数
- [ ] `contract/core.rs` `fn` 84 行 — 提取子函数
- [ ] `contract/version.rs` 两处超长：`fn` 57 行 + `fn` 59 行 — 提取子函数
- [ ] `source/git/submodule.rs` 两处超长：`fn` 54 行 + `fn` 42 行 — 提取子函数
- [ ] `stage/release.rs` 两处超长：`fmt(&self,` 51 行 + 测试函数 44 行 — 简化
- [ ] `stage/test.rs` `fn` 44 行 — 简化
- [ ] `contract/platform.rs` `default()` 55 行 — 简化
- [ ] 6 个 examples 的 `main()` 超长（45–106 行）— 提取示例逻辑到辅助函数

### 嵌套深度（code audit）

- [ ] `source/roadmap.rs` 嵌套 7 层 — 提前 return / 提取子函数
- [ ] `source/config_file.rs` 嵌套 6 层 — 提前 return
- [ ] `contract/scope.rs` 嵌套 6 层 — 提取子函数
- [ ] `contract/core.rs` 嵌套 6 层 — 提前 return
- [ ] 4 个 examples 嵌套 5-6 层 — 简化

### 圈复杂度（code audit）

- [ ] `source/roadmap.rs` `fn` 圈复杂度 14 — 提取条件分支
- [ ] `examples/contract_version.rs` `main()` 圈复杂度 12 — 提取子函数

### 模块文档覆盖率

- [ ] 24/36 文件缺少 `//!` 模块文档（覆盖率 33%）— 补全

### 超长文件

- [ ] `source/roadmap.rs` (833 行) — 拆分模块

### CI

- [ ] 缺少 CI 工作流文件（`.github/workflows/*.yml`）— 新增构建+测试流水线

### 错误处理样板

- [ ] `contract/error.rs`、`source/changelog.rs`、`source/git_tag.rs` 均手写 `Display`/`Error`/`From`，改用 `thiserror` 派生减少样板

### unsafe

- [ ] `source/changelog.rs` L76 — `transmute<'static>` 延长 `parse_changelog::parse()` 返回引用的生命周期，改用 `self_cell`、`ouroboros` 或全 owned 方案消除

### 依赖

- [ ] 引入 `thiserror = "2"` 作为 dev 依赖（非必需 feature）