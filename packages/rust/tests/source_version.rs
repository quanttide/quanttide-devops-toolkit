/// 集成测试：git 错误处理、tag 读取与版本状态。
use std::path::Path;

fn init_repo_with_tags(dir: &Path, tags: &[&str]) {
    let repo = git2::Repository::init(dir).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let tree = {
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        repo.find_tree(oid).unwrap()
    };
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
    for tag in tags {
        let target = repo.head().unwrap().target().unwrap();
        repo.tag_lightweight(tag, &repo.find_object(target, None).unwrap(), false)
            .unwrap();
    }
}

#[test]
fn test_git_error_display() {
    use quanttide_devops::source::version::VersionSourceError;

    let err = VersionSourceError::RepoOpen("/nonexistent".into());
    assert!(err.to_string().contains("无法打开仓库"));

    let err = VersionSourceError::Gix("something went wrong".into());
    assert!(err.to_string().contains("gix 错误"));
}

// ── latest_tag ────────────────────────────────────────────

#[test]
fn test_latest_tag_no_tags() {
    let d = tempfile::tempdir().unwrap();
    git2::Repository::init(d.path()).unwrap();
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "cli").unwrap(),
        None
    );
}

#[test]
fn test_latest_tag_scoped() {
    let d = tempfile::tempdir().unwrap();
    let tags = &["cli/v0.2.0", "cli/v0.1.0", "v1.0.0"];
    // 用 git2 创建真实仓库和 tag
    let repo = git2::Repository::init(d.path()).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let tree = {
        let mut idx = repo.index().unwrap();
        let oid = idx.write_tree().unwrap();
        repo.find_tree(oid).unwrap()
    };
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
    let target = repo.head().unwrap().target().unwrap();
    for tag in tags {
        repo.tag_lightweight(tag, &repo.find_object(target, None).unwrap(), false)
            .unwrap();
    }
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "cli").unwrap(),
        Some("0.2.0".into())
    );
}

#[test]
fn test_latest_tag_semver_sort() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v9.0.0", "cli/v10.0.0"]);
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "cli").unwrap(),
        Some("10.0.0".into())
    );
}

#[test]
fn test_latest_tag_unscoped_fallback() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["v1.0.0"]);
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "cli").unwrap(),
        Some("1.0.0".into())
    );
}

#[test]
fn test_latest_tag_multiple_scopes() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.2.0", "studio/v0.3.0", "cli/v0.1.0"]);
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "cli").unwrap(),
        Some("0.2.0".into())
    );
    assert_eq!(
        quanttide_devops::source::version::latest_tag(d.path(), "studio").unwrap(),
        Some("0.3.0".into())
    );
}

// ── tags_for_scope ─────────────────────────────────────────

#[test]
fn test_tags_for_scope() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.1.0", "cli/v0.2.0", "studio/v0.1.0"]);
    let tags = quanttide_devops::source::version::tags_for_scope(d.path(), "cli").unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"cli/v0.1.0".to_string()));
    assert!(tags.contains(&"cli/v0.2.0".to_string()));
}

#[test]
fn test_tags_for_scope_no_match() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["v1.0.0"]);
    assert!(
        quanttide_devops::source::version::tags_for_scope(d.path(), "cli")
            .unwrap()
            .is_empty()
    );
}

// ── verify_version ─────────────────────────────────────────

#[test]
fn test_git_verify_version() {
    let d = tempfile::tempdir().unwrap();
    let scope = scope_for_path(".");

    // 无 git 仓库 → RepoOpen 错误
    let result = quanttide_devops::source::version::verify_version(d.path(), &scope);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("无法打开仓库"));

    // 有 git 仓库无 tag → tag_version=None，config_version=Some
    init_repo_with_tags(d.path(), &[]);
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    let vs = quanttide_devops::source::version::verify_version(d.path(), &scope).unwrap();
    assert!(vs.tag_version.is_none());
    assert!(vs.config_version.is_some());

    // 打 tag 后 → 一致
    let repo = git2::Repository::open(d.path()).unwrap();
    let target = repo.head().unwrap().target().unwrap();
    repo.tag_lightweight(
        "test/v0.1.0",
        &repo.find_object(target, None).unwrap(),
        false,
    )
    .unwrap();
    let vs = quanttide_devops::source::version::verify_version(d.path(), &scope).unwrap();
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert_eq!(vs.config_version.as_deref(), Some("0.1.0"));
    assert!(vs.consistent);
}

#[test]
fn test_git_verify_version_config_no_version() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["test/v0.1.0"]);

    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
"#,
    )
    .unwrap();

    let scope = scope_for_path(".");
    let vs = quanttide_devops::source::version::verify_version(d.path(), &scope).unwrap();
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert!(
        vs.config_files
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
    assert!(vs.consistent);
}

fn scope_for_path(dir: &str) -> quanttide_devops::contract::Scope {
    quanttide_devops::contract::Scope {
        name: "test".into(),
        dir: dir.into(),
        language: quanttide_devops::contract::Language::Rust,
        build_tool: quanttide_devops::contract::BuildTool::Unknown("auto".into()),
        registry: quanttide_devops::contract::Registry::None,
        framework: String::new(),
        release: quanttide_devops::contract::StageRelease::default(),
        test_threshold: None,
        ci_workflow: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// read_config_versions — 集成测试
// ═══════════════════════════════════════════════════════════════════════

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
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
    assert!(!versions.is_empty());
    assert!(versions.iter().any(|(_, v)| v.as_deref() == Some("0.1.0")));
}

#[test]
fn test_read_config_versions_package_json_with_version() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("package.json"), r#"{"version":"1.2.3"}"#).unwrap();
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
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
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
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
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
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
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
}

#[test]
fn test_read_config_versions_yaml_empty_value() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("pubspec.yaml"), "version: \nname: test\n").unwrap();
    let versions = quanttide_devops::source::version::read_config_versions(d.path());
    assert!(
        versions
            .iter()
            .any(|(n, v)| n == "pubspec.yaml" && v.is_none())
    );
}
