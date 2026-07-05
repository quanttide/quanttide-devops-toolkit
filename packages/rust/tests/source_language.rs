use quanttide_devops::source::language::detect;

#[test]
fn test_detect_rust() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    assert_eq!(detect(d.path()), quanttide_devops::contract::Language::Rust);
}

#[test]
fn test_detect_unknown() {
    let d = tempfile::tempdir().unwrap();
    assert!(matches!(
        detect(d.path()),
        quanttide_devops::contract::Language::Unknown(_)
    ));
}

#[test]
fn test_detect_python() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    assert_eq!(
        detect(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

#[test]
fn test_detect_python_requirements() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("requirements.txt"), "").unwrap();
    assert_eq!(
        detect(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

#[test]
fn test_detect_go() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("go.mod"), "").unwrap();
    assert_eq!(detect(d.path()), quanttide_devops::contract::Language::Go);
}

#[test]
fn test_detect_dart() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
    assert_eq!(detect(d.path()), quanttide_devops::contract::Language::Dart);
}

#[test]
fn test_detect_typescript() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(
        detect(d.path()),
        quanttide_devops::contract::Language::TypeScript
    );
}
