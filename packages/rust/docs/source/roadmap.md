# source/roadmap.rs — ROADMAP 事实源封装

> 将 ROADMAP.md 解析、进度统计、格式验证封装为 toolkit 的公共能力，
> 替代 CLI 中 `plan.rs` 的手写解析。

## 定位

`source/roadmap.rs` 是"事实源"维度的 ROADMAP 实现模块。
与 `source/changelog.rs`（Keep a Changelog）、`source/git.rs`（git tag）平级。

| 模块 | 读取目标 | 格式 |
|------|---------|------|
| `source/changelog.rs` | CHANGELOG.md | Keep a Changelog |
| `source/git.rs` | Git tag | 按 scope 前缀过滤 |
| `source/roadmap.rs` | ROADMAP.md | Keep a Changelog + checkbox 任务清单 |

### 格式差异：ROADMAP vs CHANGELOG

两者格式相近但用途不同：

| 维度 | CHANGELOG | ROADMAP |
|------|-----------|---------|
| 时间方向 | 倒序（最新在上） | **正序**（近期在上） |
| 条目状态 | 已发生的事实 | `[x]` 已完成 / `[ ]` 待实施 |
| 用途 | 发布说明 | 规划跟踪 |
| 版本号 | 不带 `v` 前缀（`[0.1.0]`） | **不带** `v` 前缀（`[0.1.0]`） |

ROADMAP 独有的结构：

```markdown
## [0.2.0]       ← 版本标题

### Added         ← 分类标题（标准大小写）
- [ ] build run   ← 未完成
- [x] plan status ← 已完成
```

## 依赖

无新增依赖。纯文本解析，无需外部库。

## 核心 API

```rust
/// ROADMAP 解析结果。与 `Changelog` 对称设计，但不依赖外部解析库。
pub struct Roadmap {
    raw: String,
    versions: Vec<RoadmapVersion>,
}

/// 单个版本的规划进度。
#[derive(Debug)]
pub struct RoadmapVersion {
    pub version: String,
    pub done: usize,
    pub total: usize,
}

/// 格式验证发现的单个问题。
#[derive(Debug)]
pub struct RoadmapIssue {
    pub line: usize,
    pub scope: String,
    pub message: String,
}

impl Roadmap {
    /// 从文件路径解析 ROADMAP.md。
    pub fn from_path(path: &Path) -> Result<Self, RoadmapError>

    /// 从字符串解析 ROADMAP.md。
    pub fn from_str(s: &str) -> Result<Self, RoadmapError>

    /// 获取所有版本的规划进度。
    pub fn versions(&self) -> &[RoadmapVersion]

    /// 总已完成条目数。
    pub fn total_done(&self) -> usize

    /// 总条目数。
    pub fn total_all(&self) -> usize

    /// 验证 ROADMAP.md 格式问题（只读）。
    ///
    /// 规则：
    /// - 版本号禁止 `v` 前缀
    /// - 分类标题必须使用标准大小写（`### Added` 而非 `### added`）
    /// - checkbox 必须使用标准格式（`- [ ] ` 或 `- [x] `）
    pub fn validate(&self, scope: &str) -> Vec<RoadmapIssue>
}

/// ROADMAP 操作错误。
#[derive(Debug)]
pub enum RoadmapError {
    Io(std::io::Error),
    Parse(String),
}
```

## 与 `Changelog` 的对比

| 特性 | `Changelog` | `Roadmap` |
|------|-------------|-----------|
| 解析引擎 | `parse-changelog`（第三方） | 手写文本解析 |
| 自引用（unsafe） | 需要（`raw` + `inner` 生命周期） | **不需要** ✅—`version` 是 `String` |
| API 风格 | 按版本号索引（`["0.1.0"]`） | 按数组索引（`versions()[0]`） |
| 格式校验 | 无（依赖 parse-changelog 自有校验） | `validate()` 内置 |
| 写操作 | ❌ | ❌ |

`Roadmap` 不需要 unsafe transmute 是因为 `RoadmapVersion` 的字段都是 `String`/`usize`（自有数据），不借用 `raw`。这是和 `Changelog` 的重要结构差异。

## 与现有 CLI 函数的对等替换

```rust
// 旧（CLI plan.rs 中手写解析）
fn parse_roadmap(path: &Path) -> Result<Vec<VersionProgress>, String>
fn validate_roadmap(path: &Path, scope: &str) -> Result<Vec<Issue>, String>

// 新
let rm = Roadmap::from_path(path)?;
rm.versions()          // parse_roadmap
rm.total_done()        // 总完成数
rm.total_all()         // 总条目数
rm.validate("scope")   // validate_roadmap
```

## 测试策略

- 纯文本解析，`from_str()` 直接测试，无需文件系统
- 覆盖：空文件、单版本、多版本、v 前缀、分类大小写、checkbox 异常格式
- `validate()` 单独验证每条规则，确保只读不修改

## 迁移步骤

CLI 侧替换后 `plan.rs` 中可删除：

- `parse_roadmap()` 和 `VersionProgress` — 使用 `Roadmap::from_path().versions()`
- `validate_roadmap()` 和 `Issue` — 使用 `Roadmap::from_path().validate()`

保留在 CLI 中：

- `clean_roadmap()` — 写操作，不符合事实源定位
- `resolve_roadmap_path()` — 依赖 contract + cwd，CLI 职责
- `print_status()` — 输出格式，CLI 职责

## 不做的

- 不做 ROADMAP.md 写入/清理——属于 CLI `plan clean` 的职责
- 不做 LLM 驱动的自动修复——属于 CLI `plan doctor` 的职责
- 不做 CHANGELOG 那样的 release notes 提取——CHANGELOG 是向外看的版本记录，ROADMAP 是向内看的规划跟踪，两者用途不同，提取逻辑也不同
