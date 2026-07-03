/// 集成测试：Changelog 错误格式化。

// ═══════════════════════════════════════════════════════════════════════
// source::changelog — Display
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_changelog_error_display() {
    use quanttide_devops::source::changelog::ChangelogError;

    let err = ChangelogError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(err.to_string().contains("读取 CHANGELOG 失败"));

    let err = ChangelogError::Parse("syntax error".into());
    assert!(err.to_string().contains("解析 CHANGELOG 失败"));
}
