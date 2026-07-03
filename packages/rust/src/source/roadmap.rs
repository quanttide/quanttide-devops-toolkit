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
    /// - `# ROADMAP` 作为文档标题（必须）
    /// - `## [版本号] — 状态` 作为版本边界
    /// - `### 分类` 作为类别分组（Added / Fixed / Changed 等）
    /// - `- [ ] 描述` 为待办条目，`- [x] 描述` 为已完成
    pub fn from_str(s: &str) -> Result<Self, RoadmapError> {
        let lines: Vec<&str> = s.lines().collect();
        if lines.is_empty() {
            return Err(RoadmapError::Parse("ROADMAP 为空".into()));
        }

        // 校验第一行是否为 `# ROADMAP`
        let first = lines[0].trim();
        if first != "# ROADMAP" {
            return Err(RoadmapError::Parse(format!(
                "首行应包含 `# ROADMAP`，发现: {}",
                first
            )));
        }

        let mut versions: Vec<RoadmapVersion> = Vec::new();
        let mut current_version: Option<RoadmapVersionBuilder> = None;

        for line in &lines[1..] {
            let trimmed = line.trim();

            // 跳过空行和 blockquote
            if trimmed.is_empty() || trimmed.starts_with('>') {
                continue;
            }

            if trimmed.starts_with("## ") {
                // 新版本区块开始
                if let Some(builder) = current_version.take() {
                    versions.push(builder.build());
                }
                match parse_version_header(trimmed) {
                    Ok((version, status)) => {
                        current_version = Some(RoadmapVersionBuilder::new(version, status));
                    }
                    Err(e) => {
                        return Err(RoadmapError::Parse(format!(
                            "版本标题格式无效: {} — {}",
                            trimmed, e
                        )));
                    }
                }
            } else if let Some(ref mut builder) = current_version {
                if trimmed.starts_with("### ") {
                    // 新分类
                    let category = trimmed[4..].trim().to_string();
                    builder.add_category(category);
                } else if trimmed.starts_with("- [") && trimmed.len() > 5 {
                    // checklist 条目：`- [ ]` 或 `- [x]`
                    let completed = trimmed.as_bytes()[3] == b'x';
                    let description = trimmed[5..].trim().to_string();
                    builder.add_issue(completed, description);
                }
                // 其他行（描述文本）忽略
            }
        }

        // 收尾最后一个版本
        if let Some(builder) = current_version.take() {
            versions.push(builder.build());
        }

        if versions.is_empty() {
            return Err(RoadmapError::Parse(
                "未找到任何版本区块 (`## [x.y.z]`)".into(),
            ));
        }

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
        let mut issues = Vec::new();
        let mut line_number: usize = 1;
        let lines: Vec<&str> = self.raw.lines().collect();

        for line in &lines {
            let trimmed = line.trim();

            // 检查版本号格式（`## [0.1.0]` 或 `## [v0.1.0]`）
            if trimmed.starts_with("## [") {
                if let Some(end) = trimmed.find(']') {
                    let raw_version = &trimmed[4..end];
                    // 去 v 前缀后验证 X.Y.Z 格式
                    let clean = raw_version.strip_prefix('v').unwrap_or(raw_version);
                    let parts: Vec<&str> = clean.split('.').collect();
                    if parts.len() != 3
                        || parts
                            .iter()
                            .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
                    {
                        issues.push(RoadmapIssue {
                            line: line_number,
                            scope: scope.to_string(),
                            message: format!("版本号格式异常（期待 `X.Y.Z`）: `{}`", raw_version),
                        });
                    }
                }
            }

            // 检查分类标题的标准大小写
            if trimmed.starts_with("### ") {
                let category = trimmed[4..].trim();
                let expected = category_expected_case(category);
                if category != expected {
                    issues.push(RoadmapIssue {
                        line: line_number,
                        scope: scope.to_string(),
                        message: format!(
                            "分类标题大小写不标准: `### {}`，标准写法为 `### {}`",
                            category, expected
                        ),
                    });
                }
            }

            // 检查 checklist 格式
            if trimmed.starts_with("- [") && trimmed.len() > 5 {
                let third = trimmed.as_bytes().get(3);
                if third != Some(&b' ') && third != Some(&b'x') {
                    issues.push(RoadmapIssue {
                        line: line_number,
                        scope: scope.to_string(),
                        message: format!(
                            "checkbox 格式异常: `{}`，标准为 `- [ ]` 或 `- [x]`",
                            trimmed
                        ),
                    });
                }
            }

            line_number += 1;
        }

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
mod tests {
    use super::*;

    fn sample_roadmap() -> &'static str {
        "\
# ROADMAP

> 格式说明。

## [0.1.5] — 待实施

### Added
- [ ] item one
- [ ] item two

### Fixed
- [ ] bug fix

## [0.1.4] — 已发布

### Added
- [x] changelog module
- [x] CI workflow
"
    }

    fn mixed_roadmap() -> &'static str {
        "\
# ROADMAP

## [0.2.0] — 待实施

### Added
- [ ] big feature

## [0.1.0] — 已发布

### Added
- [x] initial
- [x] second
- [ ] third
"
    }

    // ── RoadmapVersion ─────────────────────────────────────────────

    #[test]
    fn test_version_percent() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 2,
            total: 4,
            categories: Vec::new(),
        };
        assert!((v.percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_empty() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 0,
            total: 0,
            categories: Vec::new(),
        };
        assert!((v.percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_all_done() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 3,
            total: 3,
            categories: Vec::new(),
        };
        assert!((v.percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_none_done() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 0,
            total: 5,
            categories: Vec::new(),
        };
        assert!((v.percent() - 0.0).abs() < f64::EPSILON);
    }

    // ── RoadmapProgress ────────────────────────────────────────────

    #[test]
    fn test_progress_percent() {
        let p = RoadmapProgress {
            total: 4,
            completed: 2,
        };
        assert!((p.percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_empty() {
        let p = RoadmapProgress {
            total: 0,
            completed: 0,
        };
        assert!((p.percent() - 100.0).abs() < f64::EPSILON);
    }

    // ── 解析 ────────────────────────────────────────────────────────

    #[test]
    fn test_from_str_empty() {
        let r = Roadmap::from_str("");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("为空"));
    }

    #[test]
    fn test_from_str_no_header() {
        let r = Roadmap::from_str("## [0.1.0] — test\n");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("首行"));
    }

    #[test]
    fn test_from_str_valid() {
        let r = Roadmap::from_str(sample_roadmap()).unwrap();
        let versions = r.versions();
        assert_eq!(versions.len(), 2);

        // 第一个版本 [0.1.5]
        let v1 = &versions[0];
        assert_eq!(v1.version, "0.1.5");
        assert_eq!(v1.status, "待实施");
        assert_eq!(v1.categories.len(), 2);
        assert_eq!(v1.categories[0].0, "Added");
        assert_eq!(v1.categories[0].1.len(), 2);
        assert_eq!(v1.categories[1].0, "Fixed");
        assert_eq!(v1.categories[1].1.len(), 1);

        // 条目状态
        assert!(!v1.categories[0].1[0].completed);
        assert!(!v1.categories[0].1[1].completed);
        assert!(!v1.categories[1].1[0].completed);

        // done/total
        assert_eq!(v1.total, 3);
        assert_eq!(v1.done, 0);

        // 第二个版本 [0.1.4]
        let v2 = &versions[1];
        assert_eq!(v2.version, "0.1.4");
        assert_eq!(v2.status, "已发布");
        assert_eq!(v2.total, 2);
        assert_eq!(v2.done, 2);

        // 全局统计
        assert_eq!(r.total_done(), 2);
        assert_eq!(r.total_all(), 5);
    }

    #[test]
    fn test_from_str_mixed() {
        let r = Roadmap::from_str(mixed_roadmap()).unwrap();
        let versions = r.versions();
        assert_eq!(versions.len(), 2);

        let v1 = &versions[0];
        assert_eq!(v1.version, "0.2.0");
        assert_eq!(v1.status, "待实施");
        assert_eq!(v1.total, 1);
        assert_eq!(v1.done, 0);

        let v2 = &versions[1];
        assert_eq!(v2.version, "0.1.0");
        assert_eq!(v2.status, "已发布");
        assert_eq!(v2.total, 3);
        assert_eq!(v2.done, 2);

        assert_eq!(r.total_done(), 2);
        assert_eq!(r.total_all(), 4);
    }

    #[test]
    fn test_from_path() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("ROADMAP.md");
        std::fs::write(&path, sample_roadmap()).unwrap();
        let r = Roadmap::from_path(&path).unwrap();
        assert_eq!(r.versions().len(), 2);
        assert_eq!(r.versions()[0].version, "0.1.5");
    }

    #[test]
    fn test_from_path_not_found() {
        let r = Roadmap::from_path(Path::new("/nonexistent/ROADMAP.md"));
        assert!(r.is_err());
    }

    // ── 格式验证 ────────────────────────────────────────────────────

    #[test]
    fn test_validate_valid() {
        let r = Roadmap::from_str(sample_roadmap()).unwrap();
        let issues = r.validate("test-scope");
        let msgs: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        assert!(issues.is_empty(), "预期无验证问题，发现: {:?}", msgs);
    }

    #[test]
    fn test_validate_v_prefix_allowed() {
        // v 前缀应被允许且标准化（与 CHANGELOG 统一）
        let s = "\
# ROADMAP

## [v0.1.0] — test

### Added
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        // 解析时应标准化去掉 v 前缀
        assert_eq!(r.versions()[0].version, "0.1.0");
        // validate 不应报 v 前缀相关的错误
        let issues = r.validate("scope");
        let v_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("v 前缀"))
            .collect();
        assert!(
            v_issues.is_empty(),
            "不应有 v 前缀相关验证问题: {:?}",
            v_issues
        );
    }

    #[test]
    fn test_validate_invalid_version() {
        let s = "\
# ROADMAP

## [abc] — 待实施

### Added
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.message.contains("版本号格式异常")));
    }

    #[test]
    fn test_validate_category_case() {
        let s = "\
# ROADMAP

## [0.1.0] — test

### added
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.message.contains("大小写不标准")));
    }

    #[test]
    fn test_validate_line_numbers() {
        // 用不合法版本号测试行号，v 前缀现在被允许
        let s = "\
# ROADMAP

## [abc] — test

### Added
- [ ] ok
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        assert!(!issues.is_empty());
        // 格式异常在第 3 行（1-based）
        assert_eq!(issues[0].line, 3);
        assert_eq!(issues[0].scope, "scope");
    }

    // ── parse_version_header ───────────────────────────────────────

    #[test]
    fn test_parse_version_header_normal() {
        let (v, s) = parse_version_header("## [0.1.5] — 待实施").unwrap();
        assert_eq!(v, "0.1.5");
        assert_eq!(s, "待实施");
    }

    #[test]
    fn test_parse_version_header_no_bracket() {
        let r = parse_version_header("## 0.1.5 — test");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_version_header_empty_version() {
        let r = parse_version_header("## [] — test");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_version_header_hyphen() {
        let (v, s) = parse_version_header("## [0.1.0] - released").unwrap();
        assert_eq!(v, "0.1.0");
        assert_eq!(s, "released");
    }

    #[test]
    fn test_parse_version_header_no_status() {
        let (v, s) = parse_version_header("## [0.1.0]").unwrap();
        assert_eq!(v, "0.1.0");
        assert_eq!(s, "");
    }

    // ── 覆盖率补充 ────────────────────────────────────────────────────

    #[test]
    fn test_from_str_parse_version_header_error() {
        // 通过 from_str 触发 parse_version_header 报错分支
        let s = "\
# ROADMAP

## 无方括号版本
";
        let r = Roadmap::from_str(s);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("版本标题格式无效"));
    }

    #[test]
    fn test_from_str_no_versions() {
        // 有 # ROADMAP 但无 ## [x.y.z] ，触发 versions.is_empty() 分支
        let s = "\
# ROADMAP

> 只有描述，没有版本。
";
        let r = Roadmap::from_str(s);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("未找到任何版本区块"));
    }

    #[test]
    fn test_validate_invalid_checkbox() {
        // 非标准 checkbox 格式触发 validate 的 checklist 分支
        let s = "\
# ROADMAP

## [0.1.0] — test

### Added
- [X] uppercase
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        let checkbox_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("checkbox 格式异常"))
            .collect();
        assert_eq!(checkbox_issues.len(), 1);
        assert!(checkbox_issues[0].message.contains("- [X]"));
    }

    #[test]
    fn test_validate_unknown_category() {
        // 自定义分类触发 category_expected_case 的 catch-all
        let s = "\
# ROADMAP

## [0.1.0] — test

### CustomSection
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        // 自定义分类不应报大小写问题（保持原样）
        let issues = r.validate("scope");
        // 应无任何大小写相关验证问题
        assert!(!issues.iter().any(|i| i.message.contains("大小写不标准")));
    }
}
