use std::path::Path;

fn init_repo_with_tags(dir: &Path, tags: &[&str]) {
    let repo = git2::Repository::init(dir).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let tree = {
        let mut idx = repo.index().unwrap();
        let oid = idx.write_tree().unwrap();
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

#[test]
fn test_verify_version_no_repo() {
    let d = tempfile::tempdir().unwrap();
    let result = quanttide_devops::contract::verify_version(d.path(), &scope_for_path("."));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("无法打开仓库"));
}

#[test]
fn test_verify_version_no_tag() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &[]);
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    let vs = quanttide_devops::contract::verify_version(d.path(), &scope_for_path(".")).unwrap();
    assert!(vs.tag_version.is_none());
    assert!(vs.config_version.is_some());
}

#[test]
fn test_verify_version_consistent() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["test/v0.1.0"]);
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    let vs = quanttide_devops::contract::verify_version(d.path(), &scope_for_path(".")).unwrap();
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert!(vs.consistent);
}

#[test]
fn test_verify_version_config_no_version() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["test/v0.1.0"]);
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
"#,
    )
    .unwrap();
    let vs = quanttide_devops::contract::verify_version(d.path(), &scope_for_path(".")).unwrap();
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert!(
        vs.config_files
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
    assert!(vs.consistent);
}
