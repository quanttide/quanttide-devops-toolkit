# ROADMAP — quanttide-devops-toolkit (Rust)

> 共享 SDK，提供契约、source、changelog 等基础设施。

## 技术债务

### 1. unwrap/expect 密度超标

`code audit` 检测到 250 处 unwrap/expect，密度 50.9‰（阈值 10‰）。

- [ ] `source/roadmap.rs` — 833 行，超长，含大量 unwrap
- [ ] 全局 audit：批量将可安全替换的 unwrap 改为 `?` 或 proper error handling

### 2. 超长文件

- [ ] `source/roadmap.rs` (833 行) — 拆分模块
