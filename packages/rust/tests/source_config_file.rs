/// 集成测试：source::config_file — 语言检测 + 配置文件版本读取

#[test]
fn test_read_config_versions_integration() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "version = \"0.2.0\"\n").unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(!versions.is_empty());
    assert!(versions.iter().any(|(_, v)| v.as_deref() == Some("0.1.0")));
}

#[test]
fn test_read_config_versions_package_json_with_version() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), r#"{"version":"1.2.3"}"#).unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "package.json" && v.as_deref() == Some("1.2.3"))
    );
}

#[test]
fn test_read_config_versions_package_json_empty() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), r#"{"version":""}"#).unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "package.json" && v.is_none())
    );
}

#[test]
fn test_read_config_versions_pubspec_commented() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join("pubspec.yaml"),
        "# version: 0.1.0\nname: test\n",
    )
    .unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "pubspec.yaml" && v.is_none())
    );
}

#[test]
fn test_read_config_versions_cargo_empty_version() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = ""
"#,
    )
    .unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 语言检测
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_detect_language_rust() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Rust
    );
}

#[test]
fn test_detect_language_unknown() {
    let d = tempfile::tempdir().unwrap();
    assert!(matches!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Unknown(_)
    ));
}

#[test]
fn test_detect_language_python() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

#[test]
fn test_detect_language_python_requirements() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("requirements.txt"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

#[test]
fn test_detect_language_go() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("go.mod"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Go
    );
}

#[test]
fn test_detect_language_dart() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::Dart
    );
}

#[test]
fn test_detect_language_typescript() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(
        quanttide_devops::source::config_file::detect_language(d.path()),
        quanttide_devops::contract::Language::TypeScript
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 配置文件版本读取
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_read_config_versions_yaml_empty_value() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "version: \nname: test\n").unwrap();
    let versions = quanttide_devops::source::config_file::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "pubspec.yaml" && v.is_none())
    );
}
