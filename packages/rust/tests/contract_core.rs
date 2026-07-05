use std::path::Path;

#[test]
fn test_load_from_file() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        "\
scopes:
  cli:
    dir: src/cli
    language: rust
",
    )
    .unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    assert_eq!(c.scopes.len(), 1);
    assert_eq!(c.scopes[0].name, "cli");
}

#[test]
fn test_load_file_not_found() {
    let d = tempfile::tempdir().unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(
        err.to_string().contains("读取契约文件失败") || err.to_string().contains("契约文件不存在")
    );
}

#[test]
fn test_load_invalid_yaml() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("contract.yaml"), "invalid: [").unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("解析失败") || err.to_string().contains("YAML"));
}

#[test]
fn test_load_empty_yaml() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("contract.yaml"), "").unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    assert!(c.scopes.is_empty());
}

#[test]
fn test_load_full_config() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        "\
stages:
  build:
    command: cargo build
  test:
    command: cargo test
    threshold: 80
  release:
    changelog: CHANGELOG.md
    pre_publish:
      - scripts/preflight.sh
platform:
  source_control: github
  pipeline: github_actions
  artifact_registry: crates
sources:
  version:
    type: cargo
    path: Cargo.toml
scopes:
  cli:
    dir: src/cli
    language: rust
    build_tool: cargo
    framework: clap
    registry: crates
    test_threshold: 90
  studio:
    dir: src/studio
    language: dart
    build_tool: flutter
    registry: pubdev
    release:
      changelog: src/studio/CHANGELOG.md
",
    )
    .unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    assert_eq!(c.stages.build.command.as_deref(), Some("cargo build"));
    assert_eq!(c.stages.test.threshold, 80.0);
    assert_eq!(
        c.platform.source_control,
        quanttide_devops::contract::SourceControl::Github
    );
    assert_eq!(
        c.sources.version.source_type,
        quanttide_devops::contract::SourceType::Cargo
    );
    assert_eq!(c.scopes.len(), 2);
    assert_eq!(c.scopes[0].name, "cli");
    assert_eq!(c.scopes[1].name, "studio");
}

#[test]
fn test_validate_missing_dir() {
    let c = quanttide_devops::contract::Contract::default();
    assert!(c.validate(Path::new("/tmp")).is_empty());
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        "\
scopes:
  nonexistent:
    dir: does/not/exist
",
    )
    .unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    let errors = c.validate(d.path());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("does/not/exist"));
}
