use crate::source::roadmap::{Roadmap, RoadmapIssue};

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

// ═══════════════════════════════════════════════════════════════════════
// 检查函数
// ═══════════════════════════════════════════════════════════════════════

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

/// 检查版本号格式（`## [X.Y.Z]`）。
fn check_version_headers(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let trimmed = line.trim();
            if !trimmed.starts_with("## [") {
                return None;
            }
            let end = trimmed.find(']')?;
            let raw_version = &trimmed[4..end];
            let clean = raw_version.strip_prefix('v').unwrap_or(raw_version);
            let parts: Vec<&str> = clean.split('.').collect();
            if parts.len() != 3
                || parts
                    .iter()
                    .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
            {
                Some(RoadmapIssue {
                    line: i + 1,
                    scope: scope.to_string(),
                    message: format!("版本号格式异常（期待 `X.Y.Z`）: `{}`", raw_version),
                })
            } else {
                None
            }
        })
        .collect()
}

/// 检查分类标题的标准大小写。
fn check_category_case(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
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
        })
        .collect()
}

/// 检查 checkbox 格式。
fn check_checkbox_format(lines: &[&str], scope: &str) -> Vec<RoadmapIssue> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
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
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_validate_valid() {
        let r = Roadmap::from_str(sample_roadmap()).unwrap();
        let issues = r.validate("test-scope");
        let msgs: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        assert!(issues.is_empty(), "预期无验证问题，发现: {:?}", msgs);
    }

    #[test]
    fn test_validate_v_prefix_allowed() {
        let s = "\
# ROADMAP

## [v0.1.0] — test

### Added
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        assert_eq!(r.versions()[0].version, "0.1.0");
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
        let s = "\
# ROADMAP

## [abc] — test

### Added
- [ ] ok
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        assert!(!issues.is_empty());
        assert_eq!(issues[0].line, 3);
        assert_eq!(issues[0].scope, "scope");
    }

    #[test]
    fn test_validate_invalid_checkbox() {
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
        let s = "\
# ROADMAP

## [0.1.0] — test

### CustomSection
- [ ] something
";
        let r = Roadmap::from_str(s).unwrap();
        let issues = r.validate("scope");
        assert!(!issues.iter().any(|i| i.message.contains("大小写不标准")));
    }
}
