use std::path::Path;

/// 集成测试：通过真实文件系统覆盖单元测试无法触达的 I/O 代码路径。
///
/// 测试策略：
/// - 使用 `tempfile` 创建临时目录和文件
/// - 覆盖 contract.yaml 加载、detect_language_by_files、validate
/// - 不重复单元测试已经覆盖的纯函数逻辑

// ═══════════════════════════════════════════════════════════════════════
// contract::load — 从文件系统加载
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_contract_load_from_file() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = "\
scopes:
  cli:
    dir: src/cli
    language: rust
";
    std::fs::write(dir.join("contract.yaml"), yaml).unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    assert_eq!(c.scopes.len(), 1);
    assert_eq!(c.scopes[0].name, "cli");
}

#[test]
fn test_contract_load_file_not_found() {
    let d = tempfile::tempdir().unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(
        err.to_string().contains("读取契约文件失败") || err.to_string().contains("契约文件不存在")
    );
}

#[test]
fn test_contract_load_invalid_yaml() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("contract.yaml"), "invalid: [").unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("解析失败") || err.to_string().contains("YAML"));
}

#[test]
fn test_contract_load_empty_yaml() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("contract.yaml"), "").unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    assert!(c.scopes.is_empty());
    assert_eq!(c.stages.test.threshold, 80.0);
}

// ═══════════════════════════════════════════════════════════════════════
// contract::load — 完整四维配置
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_contract_load_full_config() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = "\
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
";
    std::fs::write(dir.join("contract.yaml"), yaml).unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();

    // Stages
    assert_eq!(c.stages.build.command.as_deref(), Some("cargo build"));
    assert_eq!(c.stages.test.command.as_deref(), Some("cargo test"));
    assert_eq!(c.stages.test.threshold, 80.0);
    assert_eq!(c.stages.release.changelog, "CHANGELOG.md");
    assert_eq!(c.stages.release.pre_publish, vec!["scripts/preflight.sh"]);

    // Platforms
    assert_eq!(
        c.platform.source_control,
        quanttide_devops::contract::SourceControl::Github
    );
    assert_eq!(
        c.platform.pipeline,
        quanttide_devops::contract::Pipeline::GithubActions
    );
    assert_eq!(c.platform.artifact_registry.to_string(), "crates.io");

    // Sources
    assert_eq!(
        c.sources.version.source_type,
        quanttide_devops::contract::SourceType::Cargo
    );
    assert_eq!(c.sources.version.path.as_deref(), Some("Cargo.toml"));

    // Scopes
    assert_eq!(c.scopes.len(), 2);

    let cli = &c.scopes[0];
    assert_eq!(cli.name, "cli");
    assert_eq!(cli.dir, "src/cli");
    assert_eq!(cli.language, quanttide_devops::contract::Language::Rust);
    assert_eq!(cli.build_tool, quanttide_devops::contract::BuildTool::Cargo);
    assert_eq!(cli.framework, "clap");
    assert_eq!(cli.test_threshold, Some(90.0));

    let studio = &c.scopes[1];
    assert_eq!(studio.name, "studio");
    assert_eq!(studio.dir, "src/studio");
    assert_eq!(studio.language, quanttide_devops::contract::Language::Dart);
}

// ═══════════════════════════════════════════════════════════════════════
// detect_language_by_files — 文件系统检测
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_detect_language_by_files_rust() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::Rust
    );
}

#[test]
fn test_detect_language_by_files_unknown() {
    let d = tempfile::tempdir().unwrap();
    let lang = quanttide_devops::contract::detect_language_by_files(d.path());
    assert!(matches!(
        lang,
        quanttide_devops::contract::Language::Unknown(_)
    ));
}

// ═══════════════════════════════════════════════════════════════════════
// validate — 目录不存在
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_contract_validate_missing_dir() {
    use quanttide_devops::contract::Contract;
    let c = Contract::default();
    // 空契约 validate 应返回空
    assert!(c.validate(Path::new("/tmp")).is_empty());

    // 带 scope 但目录不存在的 validate
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = "\
scopes:
  nonexistent:
    dir: does/not/exist
";
    std::fs::write(dir.join("contract.yaml"), yaml).unwrap();
    let c = quanttide_devops::contract::load(d.path()).unwrap();
    let errors = c.validate(d.path());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("does/not/exist"));
}

// ═══════════════════════════════════════════════════════════════════════
// detect_language_by_files — 全部变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_detect_language_by_files_python() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

#[test]
fn test_detect_language_by_files_go() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("go.mod"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::Go
    );
}

#[test]
fn test_detect_language_by_files_dart() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::Dart
    );
}

#[test]
fn test_detect_language_by_files_typescript() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::TypeScript
    );
}

#[test]
fn test_detect_language_by_files_python_requirements() {
    // requirements.txt 也应检测为 Python
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("requirements.txt"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::detect_language_by_files(d.path()),
        quanttide_devops::contract::Language::Python
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SourceType::detect — 文件系统检测
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_source_type_detect_cargo() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::Cargo
    );
}

#[test]
fn test_source_type_detect_pyproject() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::Pyproject
    );
}

#[test]
fn test_source_type_detect_pubspec() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::Pubspec
    );
}

#[test]
fn test_source_type_detect_package_json() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::PackageJson
    );
}

#[test]
fn test_source_type_detect_tag_only() {
    let d = tempfile::tempdir().unwrap();
    // 没有已知配置文件 → TagOnly
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::TagOnly
    );
}

#[test]
fn test_source_type_detect_priority() {
    // 同时存在多个文件时，优先顺序：Cargo > pyproject > pubspec > package.json
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
    std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
    std::fs::write(d.path().join("package.json"), "").unwrap();
    assert_eq!(
        quanttide_devops::contract::SourceType::detect(d.path()),
        quanttide_devops::contract::SourceType::Cargo
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Registry 序列化/反序列化
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_registry_serde_roundtrip() {
    use quanttide_devops::contract::Registry;
    let cases = vec![
        (Registry::Crates, "crates"),
        (Registry::PyPI, "pypi"),
        (Registry::PubDev, "pubdev"),
        (Registry::Npm, "npm"),
        (Registry::GitHubReleases, "github_releases"),
        (Registry::Docker, "docker"),
        (Registry::None, "none"),
    ];
    for (reg, yaml) in cases {
        let serialized = serde_yaml::to_string(&reg).unwrap();
        let trimmed = serialized.trim();
        assert_eq!(trimmed, yaml, "Registry::{:?} serializes to {}", reg, yaml);
        let deserialized: Registry = serde_yaml::from_str(trimmed).unwrap();
        assert_eq!(deserialized, reg);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// validate_version — 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_validate_version_invalid_scope_chars() {
    // 空 scope → 无效
    assert!(!quanttide_devops::contract::validate_version("/v0.1.0"));
    // 含有非法字符的 scope
    assert!(!quanttide_devops::contract::validate_version(
        "bad space/v0.1.0"
    ));
}

#[test]
fn test_validate_version_empty_prerelease() {
    assert!(!quanttide_devops::contract::validate_version("v0.1.0-"));
    assert!(!quanttide_devops::contract::validate_version("v0.1.0-."));
}

// ═══════════════════════════════════════════════════════════════════════
// read_all_config_versions — 集成测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_read_all_config_versions_integration() {
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
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
    assert!(!versions.is_empty());
    // 至少有一个版本被正确读取
    assert!(versions.iter().any(|(_, v)| v.as_deref() == Some("0.1.0")));
}

// ═══════════════════════════════════════════════════════════════════════
// read_all_config_versions — JSON 版本提取
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_read_config_versions_package_json_with_version() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), r#"{"version":"1.2.3"}"#).unwrap();
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
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
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
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
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
    // 注释掉的 version 不应被提取
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "pubspec.yaml" && v.is_none())
    );
}

// ═══════════════════════════════════════════════════════════════════════
// read_all_config_versions — 空值边缘情况
// ═══════════════════════════════════════════════════════════════════════

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
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
    // version 为空字符串 → 返回 (filename, None)
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
}

#[test]
fn test_read_config_versions_yaml_empty_value() {
    let d = tempfile::tempdir().unwrap();
    // version: 后面只有空白
    std::fs::write(d.path().join("pubspec.yaml"), "version: \nname: test\n").unwrap();
    let versions = quanttide_devops::contract::read_all_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "pubspec.yaml" && v.is_none())
    );
}

// ═══════════════════════════════════════════════════════════════════════
// scope 反序列化异常 — 覆盖 ScopesVisitor::expecting
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_scope_deserialize_wrong_type() {
    // 通过 serde_yaml 触发 ScopesVisitor: scopes 不是映射而是字符串
    let yaml = "scopes: not_a_map\n";
    let result = quanttide_devops::contract::load_from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn test_scope_deserialize_type_error() {
    // 构造一个 YAML 映射，其中 scope 条目的 dir 字段类型错误
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = r#"
scopes:
  cli:
    dir:
      nested: value
"#;
    std::fs::write(dir.join("contract.yaml"), yaml).unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("解析失败") || err.to_string().contains("作用域"));
}

#[test]
fn test_scope_deserialize_missing_dir() {
    // scope 缺少必填字段 `dir`
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = r#"
scopes:
  cli:
    language: rust
"#;
    std::fs::write(dir.join("contract.yaml"), yaml).unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("dir"));
}
