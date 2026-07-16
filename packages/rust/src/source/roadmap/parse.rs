use std::path::Path;

use crate::source::roadmap::{
    Roadmap, RoadmapChecklistItem, RoadmapError, RoadmapVersion, RoadmapVersionBuilder,
};

// ═══════════════════════════════════════════════════════════════════════
// 入口
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

/// 标准化版本号：去掉 `v` 前缀。
fn normalize_version(v: &str) -> String {
    v.strip_prefix('v').unwrap_or(v).to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════

/// 解析 `## [0.1.5] — 待实施` 格式的版本标题。
fn parse_version_header(s: &str) -> Result<(String, String), String> {
    let inner = s[3..].trim();
    if !inner.starts_with('[') {
        return Err("版本号应以 `[` 开头".into());
    }
    let close_bracket = inner.find(']').ok_or("缺少 `]`".to_string())?;
    let version = inner[1..close_bracket].to_string();
    if version.is_empty() {
        return Err("版本号为空".into());
    }
    let version = version.strip_prefix('v').unwrap_or(&version).to_string();
    let rest = inner[close_bracket + 1..].trim();
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
    use crate::source::roadmap::Roadmap;

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

    // ── from_str ─────────────────────────────────────────────────

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
    fn test_from_str_header_with_suffix() {
        let s = "\
# ROADMAP — cli

## [0.1.0] — test

### Added
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        assert_eq!(r.versions().len(), 1);
        assert_eq!(r.versions()[0].version, "0.1.0");
    }

    #[test]
    fn test_from_str_valid() {
        let r = Roadmap::from_str(sample_roadmap()).unwrap();
        let versions = r.versions();
        assert_eq!(versions.len(), 2);

        let v1 = &versions[0];
        assert_eq!(v1.version, "0.1.5");
        assert_eq!(v1.status, "待实施");
        assert_eq!(v1.categories.len(), 2);
        assert_eq!(v1.categories[0].0, "Added");
        assert_eq!(v1.categories[0].1.len(), 2);
        assert_eq!(v1.categories[1].0, "Fixed");
        assert_eq!(v1.categories[1].1.len(), 1);

        assert!(!v1.categories[0].1[0].completed);
        assert!(!v1.categories[0].1[1].completed);
        assert!(!v1.categories[1].1[0].completed);

        assert_eq!(v1.total, 3);
        assert_eq!(v1.done, 0);

        let v2 = &versions[1];
        assert_eq!(v2.version, "0.1.4");
        assert_eq!(v2.status, "已发布");
        assert_eq!(v2.total, 2);
        assert_eq!(v2.done, 2);

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

    // ── from_path ────────────────────────────────────────────────

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

    // ── 边界 ─────────────────────────────────────────────────────

    #[test]
    fn test_from_str_parse_version_header_error() {
        let s = "\
# ROADMAP

## 无方括号版本
";
        let r = Roadmap::from_str(s);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("未找到任何版本区块"));
    }

    #[test]
    fn test_from_str_no_versions() {
        let s = "\
# ROADMAP

> 只有描述，没有版本。
";
        let r = Roadmap::from_str(s);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("未找到任何版本区块"));
    }

    // ── parse_version_header ─────────────────────────────────────

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
}
