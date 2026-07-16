use super::*;

// ── Error display ────────────────────────────────────────────────

#[test]
fn test_roadmap_error_display() {
    let err = RoadmapError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
    ));
    assert!(err.to_string().contains("读取 ROADMAP 失败"));

    let err = RoadmapError::Parse("bad format".into());
    assert!(err.to_string().contains("解析 ROADMAP 失败"));
}

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
    // 无方括号的版本标题被 classify_line 跳过 → 无版本区块 → 报错
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
