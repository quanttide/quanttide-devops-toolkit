# source/git.rs — Git 事实源封装

> 将 Git tag 读取、scope 过滤、版本一致性检查封装为 toolkit 的公共能力，
> 替代 CLI 中手写 `git2` 调用的 `latest_tag_for_scope()`。

## 定位

`source/git.rs` 是 `source` 维度（`SourceType::TagOnly`）的实现模块。
与 `version.rs`（配置文件版本读取）平级，两者共同覆盖"版本号从哪里读"的所有场景：

| 模块 | 读取目标 |
|------|---------|
| `version.rs` | Cargo.toml / pyproject.toml / pubspec.yaml / package.json |
| `source/git.rs` | Git tag（按 scope 前缀过滤） |

## 依赖

```toml
[dependencies]
git2 = { version = "0.19", features = ["vendored-libgit2"] }
```

## 核心 API

```rust
/// Git 操作错误。
#[derive(Debug)]
pub enum GitSourceError {
    RepoOpen(String),
    TagRead(String),
    Git2(git2::Error),
}

/// 版本一致性检查结果。
#[derive(Debug)]
pub struct VersionStatus {
    pub tag_version: Option<String>,
    pub config_version: Option<String>,
    pub consistent: bool,
    pub config_files: Vec<(String, Option<String>)>,
}

/// 获取指定 scope 的最新 tag，标准化后返回（去 v 前缀和 scope 前缀）。
pub fn latest_tag(repo_path: &Path, scope_name: &str)
    -> Result<Option<String>, GitSourceError>

/// 获取指定 scope 的所有 tag（原始格式）。
pub fn tags_for_scope(repo_path: &Path, scope_name: &str)
    -> Result<Vec<String>, GitSourceError>

/// 检查 scope 配置文件版本与最新 git tag 是否一致。
pub fn version_status(repo_path: &Path, scope: &Scope)
    -> Result<VersionStatus, GitSourceError>
```

## 行为细节

### tag 过滤规则

```
tag 列表：v1.0.0, cli/v0.1.0, cli/v0.2.0, studio/v0.1.0

查询 scope="cli" 时：
1. 精确匹配 cli/v0.2.0 → 取 cli/v0.2.0
2. 无精确匹配时，取无前缀 tag（v1.0.0）作为兜底
```

### 排序改进

CLI 当前使用字符串排序，`source/git.rs` 升级为 semver 比较：

```rust
// CLI（现状）：v10.0.0 < v9.0.0 ✗
tags.sort_by(|a, b| b.cmp(a));

// toolkit（改进）：v10.0.0 > v9.0.0 ✓
tags.sort_by(|a, b| {
    let va = parse_semver(a);
    let vb = parse_semver(b);
    vb.cmp(&va)
});
```

不引入 `semver` crate，内联 `parse_semver` 处理 X.Y.Z 数字比较。

### 与 version.rs 的关系

`version_status()` 内部调用 `read_all_config_versions()`，因为版本一致性的概念天然需要比较 git tag 和配置文件两个来源。

## 测试策略

- 用 `tempfile` 创建裸 repo，用 `git2` 打 tag 再查询
- 覆盖：无 tag、单 scope tag、多 scope tag、无前缀 tag 兜底、semver 排序（v9 vs v10）
- `version_status` 集成测试：创建带配置文件的 scope 目录 + 打对应 tag 验证一致性

## 迁移步骤

CLI 侧替换：

```rust
// 旧（在 src/contract.rs 中手写 git2 调用）
fn latest_tag_for_scope(repo_path, scope_name) -> Option<String>

// 新
use quanttide_devops::contract::source::git::latest_tag;
let tag = latest_tag(repo_path, &scope.name).ok()?;
```

## 边界

- 不做 git commit / push / fetch 等写入操作——这些属于 CLI 的 `git/` 模块
- 不做 submodule 遍历——属于 `code` 命令
- 不封装 `git2::Repository` 为全局状态——每次调用打开新 repo，避免生命周期管理
