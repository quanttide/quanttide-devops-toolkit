use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// ROADMAP 操作错误。
#[derive(Debug)]
pub enum RoadmapError {
    /// 文件读取失败。
    Io(std::io::Error),
    /// 解析失败（格式不符合预期）。
    Parse(String),
}

impl std::fmt::Display for RoadmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "读取 ROADMAP 失败: {}", e),
            Self::Parse(e) => write!(f, "解析 ROADMAP 失败: {}", e),
        }
    }
}

impl std::error::Error for RoadmapError {}

impl From<std::io::Error> for RoadmapError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 类型定义
// ═══════════════════════════════════════════════════════════════════════

/// 单个 checklist 条目。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapChecklistItem {
    /// 描述文本。
    pub description: String,
    /// 是否已勾选（`[x]`）。
    pub completed: bool,
}

/// 进度统计（用于 RoadmapProgress 和 RoadmapVersion 中的进度字段）。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapProgress {
    /// 总条目数。
    pub total: usize,
    /// 已完成条目数。
    pub completed: usize,
}

impl RoadmapProgress {
    /// 完成百分比（0.0 ~ 100.0）。无条目时返回 100.0。
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }
}

/// 单版本的规划进度。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapVersion {
    /// 版本号（如 `"0.1.5"`）。
    pub version: String,
    /// 状态标签（如 `"待实施"`、`"已发布"`）。
    pub status: String,
    /// 已完成条目数。
    pub done: usize,
    /// 总条目数。
    pub total: usize,
    /// 分类分组：`(分类名, 条目列表)`。
    /// 分类名即 `### Added` / `### Fixed` 等去掉 `### ` 前缀。
    pub categories: Vec<(String, Vec<RoadmapChecklistItem>)>,
}

impl RoadmapVersion {
    /// 完成百分比（0.0 ~ 100.0）。无条目时返回 100.0。
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.done as f64 / self.total as f64) * 100.0
    }
}

/// 格式验证发现的单个问题。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapIssue {
    /// 问题所在行号（1-based）。
    pub line: usize,
    /// 问题所属 scope（验证时传入）。
    pub scope: String,
    /// 问题描述。
    pub message: String,
}

/// 解析后的 ROADMAP.md 文档。
#[derive(Debug, Clone, PartialEq)]
pub struct Roadmap {
    /// 原始文本（保障引用有效性，但当前未使用；保留以便未来扩展）。
    #[allow(dead_code)]
    raw: String,
    /// 所有版本区块（自上而下 = 最新优先）。
    versions: Vec<RoadmapVersion>,
}

// ═══════════════════════════════════════════════════════════════════════
// 解析
// ═══════════════════════════════════════════════════════════════════════

impl Roadmap {
    /// 从文件路径解析 ROADMAP.md。
    pub fn from_path(path: &Path) -> Result<Self, RoadmapError> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_str(&raw)
    }

    /// 从字符串解析 ROADMAP.md。
    ///
    /// 格式约定（Keep a Changelog 变体）：
    /// - `# ROADMAP` 作为文档标题（首行可以带后缀，如 `# ROADMAP — cli`）
    /// - `## [版本号] — 状态` 作为版本边界
    /// - `### 分类` 作为类别分组（Added / Fixed / Changed 等）
    /// - `- [ ] 描述` 为待办条目，`- [x] 描述` 为已完成
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, RoadmapError> {
        let lines: Vec<&str> = s.lines().collect();
        validate_first_line(lines.first().copied())?;
        let versions = parse_versions(&lines[1..])?;
        Ok(Self {
            raw: s.to_string(),
            versions,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 公共访问器
// ═══════════════════════════════════════════════════════════════════════

impl Roadmap {
    /// 获取所有版本的规划进度。
    pub fn versions(&self) -> &[RoadmapVersion] {
        &self.versions
    }

    /// 总已完成条目数。
    pub fn total_done(&self) -> usize {
        self.versions.iter().map(|v| v.done).sum()
    }

    /// 总条目数。
    pub fn total_all(&self) -> usize {
        self.versions.iter().map(|v| v.total).sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 格式验证
// ═══════════════════════════════════════════════════════════════════════

impl Roadmap {
    /// 验证 ROADMAP.md 格式规范。
    ///
    /// 规则：
    /// - 版本号必须为纯数字 X.Y.Z
    /// - 分类标题必须使用标准大小写（`### Added` 而非 `### added`）
    /// - checkbox 必须使用标准格式（`- [ ]` 或 `- [x]`）
    ///
    /// `scope` 参数用于标记问题所属范围（通常传入 scope name）。
    pub fn validate(&self, scope: &str) -> Vec<RoadmapIssue> {
        let lines: Vec<&str> = self.raw.lines().collect();
        let mut issues = Vec::new();
        issues.extend(check_version_headers(&lines, scope));
        issues.extend(check_category_case(&lines, scope));
        issues.extend(check_checkbox_format(&lines, scope));
        issues
    }
}

/// 返回分类的标准大小写形式。未知分类保持原样返回。
fn category_expected_case(s: &str) -> String {
    match s {
        "added" | "Added" => "Added".into(),
        "changed" | "Changed" => "Changed".into(),
        "deprecated" | "Deprecated" => "Deprecated".into(),
        "removed" | "Removed" => "Removed".into(),
        "fixed" | "Fixed" => "Fixed".into(),
        "security" | "Security" => "Security".into(),
        _ => s.to_string(),
    }
}

/// 检查版本号格式（`## [X.Y.Z]`）。非版本号行自动返回空 vec。
fn check_version_headers(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines.iter().enumerate().filter_map(|(i, line)| {
        let trimmed = line.trim();
        if !trimmed.starts_with("## [") {
            return None;
        }
        let end = trimmed.find(']')?;
        let raw_version = &trimmed[4..end];
        let clean = raw_version.strip_prefix('v').unwrap_or(raw_version);
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() != 3
            || parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
        {
            Some(RoadmapIssue {
                line: i + 1,
                scope: scope.to_string(),
                message: format!("版本号格式异常（期待 `X.Y.Z`）: `{}`", raw_version),
            })
        } else {
            None
        }
    }).collect()
}

/// 检查分类标题的标准大小写。非分类行自动返回空 vec。
fn check_category_case(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines.iter().enumerate().filter_map(|(i, line)| {
        let trimmed = line.trim();
        if !trimmed.starts_with("### ") {
            return None;
        }
        let category = trimmed.strip_prefix("### ").unwrap_or("").trim();
        let expected = category_expected_case(category);
        if category != expected {
            Some(RoadmapIssue {
                line: i + 1,
                scope: scope.to_string(),
                message: format!(
                    "分类标题大小写不标准: `### {}`，标准写法为 `### {}`",
                    category, expected
                ),
            })
        } else {
            None
        }
    }).collect()
}

/// 检查 checkbox 格式。非 checklist 行自动返回空 vec。
fn check_checkbox_format(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines.iter().enumerate().filter_map(|(i, line)| {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") || trimmed.len() <= 5 {
            return None;
        }
        let third = trimmed.as_bytes().get(3);
        if third != Some(&b' ') && third != Some(&b'x') {
            Some(RoadmapIssue {
                line: i + 1,
                scope: scope.to_string(),
                message: format!(
                    "checkbox 格式异常: `{}`，标准为 `- [ ]` 或 `- [x]`",
                    trimmed
                ),
            })
        } else {
            None
        }
    }).collect()
}

// ═══════════════════════════════════════════════════════════════════════
// 行分类器
// ═══════════════════════════════════════════════════════════════════════

/// 行类型，用于降低解析循环的嵌套深度。
enum LineKind<'a> {
    /// `## [version] — status`
    VersionHeader(&'a str, &'a str),
    /// `### CategoryName`
    Category(&'a str),
    /// `- [ ] desc` 或 `- [x] desc`
    Checklist { completed: bool, desc: &'a str },
    /// 应跳过的行（空行、blockquote）
    Skip,
}

/// 对一行按 ROADMAP 格式分类。
fn classify_line(line: &str) -> LineKind {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('>') {
        return LineKind::Skip;
    }
    if trimmed.starts_with("## ") {
        let inner = trimmed[3..].trim();
        if let Some(end) = inner.find(']') {
            let version = &inner[1..end];
            let rest = inner[end + 1..].trim();
            let status = rest
                .strip_prefix('—')
                .or_else(|| rest.strip_prefix('-'))
                .map(|s| s.trim())
                .unwrap_or("");
            return LineKind::VersionHeader(version, status);
        }
    }
    if trimmed.starts_with("### ") {
        return LineKind::Category(trimmed.strip_prefix("### ").unwrap_or("").trim());
    }
    if trimmed.starts_with("- [") && trimmed.len() > 5 {
        let completed = trimmed.as_bytes()[3] == b'x';
        return LineKind::Checklist {
            completed,
            desc: trimmed[5..].trim(),
        };
    }
    LineKind::Skip
}

// ═══════════════════════════════════════════════════════════════════════
// 解析循环
// ═══════════════════════════════════════════════════════════════════════

/// 校验第一行是否以 `# ROADMAP` 开头。
fn validate_first_line(first: Option<&str>) -> Result<(), RoadmapError> {
    match first {
        None => Err(RoadmapError::Parse("ROADMAP 为空".into())),
        Some(s) if s.trim().starts_with("# ROADMAP") => Ok(()),
        Some(s) => Err(RoadmapError::Parse(format!(
            "首行应包含 `# ROADMAP`，发现: {}",
            s.trim()
        ))),
    }
}

/// 解析版本区块，返回所有版本（自上而下 = 最新优先）。
fn parse_versions(lines: &[&str]) -> Result<Vec<RoadmapVersion>, RoadmapError> {
    let mut versions: Vec<RoadmapVersion> = Vec::new();
    let mut current_version: Option<RoadmapVersionBuilder> = None;

    for line in lines {
        match classify_line(line) {
            LineKind::Skip => {}
            LineKind::VersionHeader(version, status) => {
                if let Some(builder) = current_version.take() {
                    versions.push(builder.build());
                }
                current_version = Some(RoadmapVersionBuilder::new(
                    normalize_version(version),
                    status.to_string(),
                ));
            }
            LineKind::Category(name) => {
                if let Some(ref mut builder) = current_version {
                    builder.add_category(name.to_string());
                }
            }
            LineKind::Checklist { completed, desc } => {
                if let Some(ref mut builder) = current_version {
                    builder.add_issue(completed, desc.to_string());
                }
            }
        }
    }

    if let Some(builder) = current_version.take() {
        versions.push(builder.build());
    }

    if versions.is_empty() {
        return Err(RoadmapError::Parse(
            "未找到任何版本区块 (`## [x.y.z]`)".into(),
        ));
    }

    Ok(versions)
}

/// 标准化版本号：去掉 `v` 前缀（与 CHANGELOG 的 parse-changelog 行为一致）。
fn normalize_version(v: &str) -> String {
    v.strip_prefix('v').unwrap_or(v).to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// 内部构建器
// ═══════════════════════════════════════════════════════════════════════

struct RoadmapVersionBuilder {
    version: String,
    status: String,
    categories: Vec<(String, Vec<RoadmapChecklistItem>)>,
}

impl RoadmapVersionBuilder {
    fn new(version: String, status: String) -> Self {
        Self {
            version,
            status,
            categories: Vec::new(),
        }
    }

    fn add_category(&mut self, name: String) {
        self.categories.push((name, Vec::new()));
    }

    fn add_issue(&mut self, completed: bool, description: String) {
        if let Some(last) = self.categories.last_mut() {
            last.1.push(RoadmapChecklistItem {
                description,
                completed,
            });
        }
        // 没有打开的分类时不处理（格式异常但容错）
    }

    fn build(self) -> RoadmapVersion {
        let total: usize = self.categories.iter().map(|(_, items)| items.len()).sum();
        let done: usize = self
            .categories
            .iter()
            .flat_map(|(_, items)| items)
            .filter(|i| i.completed)
            .count();

        RoadmapVersion {
            version: self.version,
            status: self.status,
            done,
            total,
            categories: self.categories,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════

/// 解析 `## [0.1.5] — 待实施` 格式的版本标题。
fn parse_version_header(s: &str) -> Result<(String, String), String> {
    // s 形如 `## [0.1.5] — 待实施`，已 trim
    let inner = s[3..].trim(); // 去掉 `## `
    if !inner.starts_with('[') {
        return Err("版本号应以 `[` 开头".into());
    }
    let close_bracket = inner.find(']').ok_or("缺少 `]`".to_string())?;
    let version = inner[1..close_bracket].to_string();
    if version.is_empty() {
        return Err("版本号为空".into());
    }
    // 标准化：去掉 v 前缀（与 CHANGELOG 的 parse-changelog 行为一致）
    let version = version.strip_prefix('v').unwrap_or(&version).to_string();
    let rest = inner[close_bracket + 1..].trim();
    // 分隔符可以是 ` — `、` - `、` —`、` -` 等
    let status = if let Some(pos) = rest.find('—') {
        rest[pos + 3..].trim().to_string()
    } else if let Some(pos) = rest.find('-') {
        rest[pos + 1..].trim().to_string()
    } else {
        rest.to_string()
    };

    Ok((version, status))
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[path = "roadmap_tests.rs"]
mod tests;
