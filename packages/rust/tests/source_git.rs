/// 集成测试：git 错误处理与版本状态。

// ═══════════════════════════════════════════════════════════════════════
// source::git — 版本状态检查
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_git_error_display() {
    use quanttide_devops::source::git::GitSourceError;

    let err = GitSourceError::RepoOpen("/nonexistent".into());
    assert!(err.to_string().contains("无法打开仓库"));

    // 通过真实的 git 操作失败获取 git2::Error
    if let Err(git_err) = git2::Repository::open("/nonexistent") {
        let from_git: GitSourceError = git_err.into();
        assert!(from_git.to_string().contains("git2 错误"));
    }
}

#[test]
fn test_git_from_impl() {
    use quanttide_devops::source::git::GitSourceError;
    // 验证 From<git2::Error> 实现
    if let Err(git_err) = git2::Repository::open("/nonexistent") {
        let err: GitSourceError = git_err.into();
        assert!(matches!(err, GitSourceError::Git2(_)));
    }
}

#[test]
fn test_git_version_status() {
    let d = tempfile::tempdir().unwrap();
    let scope = quanttide_devops::contract::Scope {
        name: "test".into(),
        dir: ".".into(),
        language: quanttide_devops::contract::Language::Rust,
        build_tool: quanttide_devops::contract::BuildTool::Unknown("auto".into()),
        registry: quanttide_devops::contract::Registry::None,
        framework: String::new(),
        release: quanttide_devops::contract::StageRelease::default(),
        test_threshold: None,
        ci_workflow: None,
    };

    // 无 git 仓库 → RepoOpen 错误
    let result = quanttide_devops::source::git::version_status(d.path(), &scope);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("无法打开仓库"));

    // 有 git 仓库无 tag → tag_version=None，config_version=Some
    let repo = git2::Repository::init(d.path()).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let tree = {
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        repo.find_tree(oid).unwrap()
    };
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    let vs = quanttide_devops::source::git::version_status(d.path(), &scope).unwrap();
    assert!(vs.tag_version.is_none());
    assert!(vs.config_version.is_some());

    // 打 tag 后 → 一致
    repo.tag_lightweight(
        "test/v0.1.0",
        &repo
            .find_object(repo.head().unwrap().target().unwrap(), None)
            .unwrap(),
        false,
    )
    .unwrap();
    let vs = quanttide_devops::source::git::version_status(d.path(), &scope).unwrap();
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert_eq!(vs.config_version.as_deref(), Some("0.1.0"));
    assert!(vs.consistent);
}

// ═══════════════════════════════════════════════════════════════════════
// source::git — 版本状态：tag 存在但配置文件无版本
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_git_version_status_config_no_version() {
    // 有 tag 但配置文件版本为空 → consistent=true（None 被视为一致）
    let d = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(d.path()).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let tree = {
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        repo.find_tree(oid).unwrap()
    };
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
    repo.tag_lightweight(
        "test/v0.1.0",
        &repo
            .find_object(repo.head().unwrap().target().unwrap(), None)
            .unwrap(),
        false,
    )
    .unwrap();

    // 创建 Cargo.toml 但版本号缺失
    std::fs::write(
        d.path().join("Cargo.toml"),
        r#"[package]
name = "test"
"#,
    )
    .unwrap();

    let scope = quanttide_devops::contract::Scope {
        name: "test".into(),
        dir: ".".into(),
        language: quanttide_devops::contract::Language::Rust,
        build_tool: quanttide_devops::contract::BuildTool::Unknown("auto".into()),
        registry: quanttide_devops::contract::Registry::None,
        framework: String::new(),
        release: quanttide_devops::contract::StageRelease::default(),
        test_threshold: None,
        ci_workflow: None,
    };
    let vs = quanttide_devops::source::git::version_status(d.path(), &scope).unwrap();
    // tag 有版本，Cargo.toml 无版本 → 文件列表应包含 (Cargo.toml, None)
    assert_eq!(vs.tag_version.as_deref(), Some("0.1.0"));
    assert!(
        vs.config_files
            .iter()
            .any(|(n, v)| n == "Cargo.toml" && v.is_none())
    );
    // None 被视为一致 → consistent=true
    assert!(vs.consistent);
}
