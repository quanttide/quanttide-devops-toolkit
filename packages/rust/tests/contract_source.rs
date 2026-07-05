use quanttide_devops::contract::SourceType;

#[test]
fn test_source_type_detect_cargo() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::Cargo);
}

#[test]
fn test_source_type_detect_pyproject() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::Pyproject);
}

#[test]
fn test_source_type_detect_pubspec() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::Pubspec);
}

#[test]
fn test_source_type_detect_package_json() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::PackageJson);
}

#[test]
fn test_source_type_detect_tag_only() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::TagOnly);
}

#[test]
fn test_source_type_detect_priority() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(SourceType::detect(d.path()), SourceType::Cargo);
}
