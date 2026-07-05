# source/changelog.rs — CHANGELOG 事实源封装

> 将 CHANGELOG 解析、release notes 提取、版本存在性校验封装为 toolkit 的公共能力，
> 替代 CLI 中手写正则解析的 `extract_notes()` 和 `precheck_version_changelog()`。

## 定位

`source/changelog.rs` 是"事实源"维度的 CHANGELOG 实现模块。
与 `source/version.rs`（git tag）、`contract/version.rs`（配置文件版本）平级
共同覆盖"版本发布记录从哪里读"的场景：

| 模块 | 读取目标 | 依赖 |
|------|---------|------|
| `contract/version.rs` | Cargo.toml / pyproject.toml / ... | 无 |
| `source/version.rs` | Git tag 版本 | `gix` + `semver` |
| `source/changelog.rs` | CHANGELOG.md（Keep a Changelog 格式） | `parse-changelog` |

## 选型：parse-changelog

对比 `keep-a-changelog`（全规格读写）和 `parse-changelog`（只读轻量）：

| 维度 | parse-changelog | keep-a-changelog |
|------|----------------|-----------------|
| 下载量 | 274k | 40k |
| 维护者 | taiki-e（Rust 生态知名） | 个人维护 |
| 最后更新 | 2026-06-07 | 2024-07-10 |
| 依赖链 | 零额外依赖（关闭 default-features） | chrono + eyre + regex + semver + ... |
| API | `parse(str)` → `Releases`（数组+索引） | Changelog 结构体 + Builder |
| 读写 | 只读 ✅ | 读写 |

结论：`parse-changelog`。toolkit 的"事实源"定位只需要读，不需要写。

## 依赖

```toml
[dependencies]
parse-changelog = { version = "0.6", default-features = false }
```

## 核心 API

```rust
/// CHANGELOG 解析结果。`release_notes` 和 `contains_version` 两个便捷方法。
pub struct Changelog {
    inner: parse_changelog::Releases,
}

impl Changelog {
    /// 从文件路径解析 CHANGELOG。
    pub fn from_path(path: &Path) -> Result<Self, ChangelogError>

    /// 从字符串解析 CHANGELOG。
    pub fn from_str(s: &str) -> Result<Self, ChangelogError>

    /// 获取指定版本的 release notes（用于 GitHub Release body）。
    pub fn release_notes(&self, version: &str) -> Option<&str>

    /// 检查指定版本是否存在于 CHANGELOG 中。
    pub fn contains_version(&self, version: &str) -> bool

    /// 获取最新发布的版本号。
    pub fn latest_version(&self) -> Option<&str>

    /// 获取所有版本号列表。
    pub fn versions(&self) -> Vec<&str>
}

/// CHANGELOG 操作错误。
#[derive(Debug)]
pub enum ChangelogError {
    Io(std::io::Error),
    Parse(String),
}
```

## 行为细节

### 版本号匹配

`parse-changelog` 默认支持 semver 格式版本号。CLI 中的版本格式为 `v0.1.0`（带 `v` 前缀），
`parse-changelog` 默认支持 `v` 前缀，可直接匹配。

对于 scope 前缀版本（`cli/v0.1.0`），需要从版本号中提取 scope 后的 semver 部分再查询。

### 与现有函数的对等替换

```rust
// 旧（CLI release/util.rs 中手写）
fn extract_notes(version: &str, changelog_path: &Path) -> Option<String>
fn precheck_version_changelog(version: &str, changelog_path: &Path) -> Vec<String>

// 新
let changelog = Changelog::from_path(path)?;
changelog.release_notes("0.1.0")       // extract_notes
changelog.contains_version("0.1.0")    // precheck: true/false
changelog.latest_version()             // 最新版本
```

## 测试策略

- 通过构造 CHANGELOG.md 字符串验证解析输出
- 覆盖：标准格式、带 `v` 前缀、带 scope 前缀、Unreleased 段落、空文件、非法格式
- 不需要真实文件系统（`from_str` 直接测试）

## 迁移步骤

CLI 侧替换后可以删除：

- `release/util.rs` 中的 `extract_notes()` — 使用 `Changelog::release_notes()`
- `release/util.rs` 中的 `precheck_version_changelog()` — 使用 `Changelog::contains_version()`

## 不做的

- 不做 CHANGELOG 生成/写入——属于 CLI `release publish` 中 `ensure_changelog()` 的职责
- 不做格式校验/Lint——有专门的 `notabene` crate 做这个
- 不做 conventional commits 解析——属于不同的规范体系
