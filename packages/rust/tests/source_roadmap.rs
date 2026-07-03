/// 集成测试：ROADMAP.md 错误处理、进度统计、格式验证。

// ═══════════════════════════════════════════════════════════════════════
// source::roadmap — Error display
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_roadmap_error_display() {
    use quanttide_devops::source::roadmap::RoadmapError;

    let err = RoadmapError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(err.to_string().contains("读取 ROADMAP 失败"));

    let err = RoadmapError::Parse("bad format".into());
    assert!(err.to_string().contains("解析 ROADMAP 失败"));
}

// ═══════════════════════════════════════════════════════════════════════
// source::roadmap — End-to-end parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_roadmap_parse_real_file() {
    use std::path::Path;

    let r = quanttide_devops::source::roadmap::Roadmap::from_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ROADMAP.md"
    )))
    .unwrap();

    let versions = r.versions();
    // 至少包含 0.1.5 和 0.1.4 两个版本
    assert!(versions.len() >= 2, "至少应有 2 个版本");
    assert_eq!(versions[0].version, "0.1.5");
    assert_eq!(versions[0].status, "已实施");
    assert_eq!(versions[1].version, "0.1.4");

    // 0.1.5 所有条目都已勾选（刚刚实现完毕）
    assert_eq!(versions[0].done, versions[0].total);
    assert!((versions[0].percent() - 100.0).abs() < f64::EPSILON);

    // 0.1.4 已全部完成
    assert_eq!(versions[1].done, versions[1].total);
    assert!((versions[1].percent() - 100.0).abs() < f64::EPSILON);

    // 全局统计一致性
    assert_eq!(
        r.total_done(),
        r.versions().iter().map(|v| v.done).sum::<usize>()
    );
    assert_eq!(
        r.total_all(),
        r.versions().iter().map(|v| v.total).sum::<usize>()
    );

    // 格式验证通过
    let issues = r.validate("rust");
    assert!(issues.is_empty(), "ROADMAP 格式验证应通过: {:?}", issues);
}

// ═══════════════════════════════════════════════════════════════════════
// source::roadmap — Version percent
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_roadmap_version_percent() {
    use quanttide_devops::source::roadmap::RoadmapVersion;

    let v = RoadmapVersion {
        version: "0.1.0".into(),
        status: "test".into(),
        done: 3,
        total: 10,
        categories: Vec::new(),
    };
    assert!((v.percent() - 30.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════
// source::roadmap — v 前缀标准化（与 CHANGELOG 统一）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_roadmap_v_prefix_normalized() {
    use quanttide_devops::source::roadmap::Roadmap;

    let s = "\
# ROADMAP

## [v0.1.0] — test

### Added
- [x] done
";
    let r = Roadmap::from_str(s).unwrap();
    // v 前缀应在解析时被标准化去除
    assert_eq!(r.versions()[0].version, "0.1.0");
    // validate 仍能识别版本号格式
    let issues = r.validate("cli");
    assert!(issues.is_empty(), "v 前缀不应导致验证错误: {:?}", issues);
}

// ═══════════════════════════════════════════════════════════════════════
// source::roadmap — Malformed file detection
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_roadmap_malformed_no_header() {
    use quanttide_devops::source::roadmap::Roadmap;
    use std::path::Path;

    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/roadmap-malformed.md"
    ));
    let r = Roadmap::from_path(path);
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("首行"));
}
