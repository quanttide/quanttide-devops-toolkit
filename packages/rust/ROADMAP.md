# ROADMAP — quanttide-devops-toolkit (Rust)

> 共享 SDK，提供契约、source、changelog 等基础设施。

## 技术债务

### 超长文件

- [ ] `source/roadmap.rs` (833 行) — 拆分模块

### 错误处理样板

- [ ] `contract/error.rs`、`source/changelog.rs`、`source/git_tag.rs` 均手写 `Display`/`Error`/`From`，改用 `thiserror` 派生减少样板

### unsafe

- [ ] `source/changelog.rs` L76 — `transmute<'static>` 延长 `parse_changelog::parse()` 返回引用的生命周期，改用 `self_cell`、`ouroboros` 或全 owned 方案消除

### 依赖

- [ ] 引入 `thiserror = "2"` 作为 dev 依赖（非必需 feature）
