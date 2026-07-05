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

// ═══════════════════════════════════════════════════════════════════════
// auto_detect — 文件系统扫描
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_auto_detect_empty_dir() {
    let d = tempfile::tempdir().unwrap();
    let c = quanttide_devops::contract::Contract::auto_detect(d.path());
    assert!(c.scopes.is_empty());
}

#[test]
fn test_auto_detect_root_only() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "[package]\n").unwrap();
    let c = quanttide_devops::contract::Contract::auto_detect(d.path());
    assert!(!c.scopes.is_empty());
    assert!(c.scopes.iter().any(|s| s.name == "(root)"));
}

#[test]
fn test_auto_detect_with_packages() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("packages/fubar")).unwrap();
    std::fs::write(d.path().join("packages/fubar/Cargo.toml"), "[package]\n").unwrap();
    std::fs::create_dir_all(d.path().join("src/cli")).unwrap();
    std::fs::write(d.path().join("src/cli/Cargo.toml"), "[package]\n").unwrap();
    let c = quanttide_devops::contract::Contract::auto_detect(d.path());
    let names: Vec<&str> = c.scopes.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"fubar"));
    assert!(names.contains(&"cli"));
}

#[test]
fn test_auto_detect_skips_unknown_lang_dirs() {
    // 子目录无可识别配置文件 → 应跳过（覆盖 line 118 continue）
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("packages/unknown")).unwrap();
    // 在根目录放一个可识别文件让 (root) scope 出现
    std::fs::write(d.path().join("Cargo.toml"), "[package]\n").unwrap();
    let c = quanttide_devops::contract::Contract::auto_detect(d.path());
    // packages/unknown 应被跳过，只有 (root) scope
    let names: Vec<&str> = c.scopes.iter().map(|s| s.name.as_str()).collect();
    assert!(!names.contains(&"unknown"));
    assert!(names.contains(&"(root)"));
}

// ═══════════════════════════════════════════════════════════════════════
// load_or_default — 兜底流程
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_load_or_default_with_contract_file() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        "scopes:\n  cli:\n    dir: .\n    language: rust\n",
    )
    .unwrap();
    let c = quanttide_devops::contract::load_or_default(d.path());
    assert_eq!(c.scopes.len(), 1);
    assert_eq!(c.scopes[0].name, "cli");
}

#[test]
fn test_load_or_default_fallback() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "[package]\n").unwrap();
    // 无 .quanttide/devops/contract.yaml，应 fallback 到 auto_detect
    let c = quanttide_devops::contract::load_or_default(d.path());
    assert!(!c.scopes.is_empty());
}
