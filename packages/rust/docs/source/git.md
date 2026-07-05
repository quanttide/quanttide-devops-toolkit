# source/git.rs — Git 事实源封装

> 将 Git tag 读取、scope 过滤、版本一致性检查封装为 toolkit 的公共能力。
> 采用 **gix 优先，git2 兜底**的混合策略。

## 定位

`source/git.rs` 是 `source` 维度（`SourceType::TagOnly`）的实现模块。
与 `version.rs`（配置文件版本读取）平级，两者共同覆盖"版本号从哪里读"的所有场景：

| 模块 | 读取目标 |
|------|---------|
| `version.rs` | Cargo.toml / pyproject.toml / pubspec.yaml / package.json |
| `source/git.rs` | Git tag（按 scope 前缀过滤） |

## 混合策略

`source/git.rs` 只做**读操作**（tag 列表、引用遍历、repo 打开），全部由 `gix` 实现。
git2 仅在与 toolkit 的 `git/` 写模块整合时引入——当前阶段保持单一 gix 依赖。

| 操作 | 使用库 | 原因 |
|------|--------|------|
| repo 打开 | `gix` | 快 14x（1ms vs 14ms）|
| tag 列表 | `gix` | `repo.references()?.prefixed("refs/tags")` |
| semver 排序 | Rust std | 内联 `parse_semver`，不依赖 git 库 |

## 依赖

```toml
[dependencies]
gix = "0.69"
```

`git2` 不直接引入——当前 scope 内全部操作 gix 都能覆盖。
未来 toolkit 扩展写操作（commit / tag / push）时，按需追加 `git2`。

## 核心 API

```rust
/// Git 操作错误。
#[derive(Debug)]
pub enum GitSourceError {
    RepoOpen(String),
    TagRead(String),
    Gix(gix::Error),
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

## gix 实现要点

基于 `gix 0.69` API，标签读取的典型实现：

```rust
pub fn tags_for_scope(repo_path: &Path, scope_name: &str)
    -> Result<Vec<String>, GitSourceError>
{
    let repo = gix::open(repo_path).map_err(|e| GitSourceError::RepoOpen(e.to_string()))?;
    let refs = repo.references().map_err(|e| GitSourceError::TagRead(e.to_string()))?;
    let iter = refs.prefixed("refs/tags")
        .map_err(|e| GitSourceError::TagRead(e.to_string()))?;

    let tags: Vec<String> = iter
        .filter_map(|r| r.ok())
        .filter(|r| {
            let name = r.name().as_bstr().to_string_lossy();
            let short = name.strip_prefix("refs/tags/").unwrap_or(&name);
            short.starts_with(scope_name) || !short.contains('/')
        })
        .map(|r| {
            let name = r.name().as_bstr().to_string_lossy();
            name.strip_prefix("refs/tags/").unwrap_or(&name).to_string()
        })
        .collect();

    Ok(tags)
}
```

### 为什么不直接用 git2

`source/git.rs` 是 toolkit 中被最频繁调用的模块之一（每个 `status` 命令都走一遍）。
用 gix 替代 git2，每次调用节省 ~13ms repo 打开时间。实测数据：

| 操作 | git2 | gix | 收益 |
|------|------|-----|------|
| open + tags | 15ms | 2ms | CLI status 快 13ms |
| open + branches | 14.5ms | 1.5ms | CLI code status 快 13ms |
| open + HEAD | 14.3ms | 1.6ms | CLI doctor 快 12.7ms |

对于 "打开一次、查一个值、退出" 的 CLI 模型，收益是用户可感知的。

## 测试策略

- 用 `tempfile` 创建裸 repo，用 CLI `git tag` 打标签再通过 gix 查询
- 覆盖：无 tag、单 scope tag、多 scope tag、无前缀 tag 兜底、semver 排序（v9 vs v10）
- `version_status` 集成测试：创建带配置文件的 scope 目录 + 打对应 tag 验证一致性

## 迁移步骤

```rust
// 旧（在 src/contract.rs 中手写 git2 调用）
fn latest_tag_for_scope(repo_path, scope_name) -> Option<String>

// 新
use quanttide_devops::contract::source::git::latest_tag;
let tag = latest_tag(repo_path, &scope.name).ok()?;
```

## 边界

- **不做写操作**——git commit / tag 创建 / push 等属于 `git/` 模块，走 git2
- **不做 submodule 遍历**——属于 `code` 命令
- **不封装全局 repo 状态**——每次调用打开新 repo，gix 的 1ms open 成本可忽略
- **不和 git2 混用同一 repo handle**——读完 gix 关闭，写时重新用 git2 打开
