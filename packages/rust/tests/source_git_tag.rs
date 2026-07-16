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

#[test]
fn test_tag_error_display() {
    use quanttide_devops::source::git::tag::TagError;
    let err = TagError::RepoOpen("/nonexistent".into());
    assert!(err.to_string().contains("无法打开仓库"));
    let err = TagError::Gix("something went wrong".into());
    assert!(err.to_string().contains("gix 错误"));
}

#[test]
fn test_latest_tag_no_tags() {
    let d = tempfile::tempdir().unwrap();
    git2::Repository::init(d.path()).unwrap();
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "cli").unwrap(),
        None
    );
}

#[test]
fn test_latest_tag_scoped() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.2.0", "cli/v0.1.0", "v1.0.0"]);
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "cli").unwrap(),
        Some("cli/v0.2.0".into())
    );
}

#[test]
fn test_latest_tag_semver_sort() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v9.0.0", "cli/v10.0.0"]);
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "cli").unwrap(),
        Some("cli/v10.0.0".into())
    );
}

#[test]
fn test_latest_tag_unscoped_fallback() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["v1.0.0"]);
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "cli").unwrap(),
        Some("v1.0.0".into())
    );
}

#[test]
fn test_latest_tag_multiple_scopes() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.2.0", "studio/v0.3.0", "cli/v0.1.0"]);
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "cli").unwrap(),
        Some("cli/v0.2.0".into())
    );
    assert_eq!(
        quanttide_devops::source::git::tag::latest_tag(d.path(), "studio").unwrap(),
        Some("studio/v0.3.0".into())
    );
}

#[test]
fn test_latest_version() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.2.0", "cli/v0.1.0", "v1.0.0"]);
    assert_eq!(
        quanttide_devops::source::git::tag::latest_version(d.path(), "cli").unwrap(),
        Some("0.2.0".into())
    );
    assert_eq!(
        quanttide_devops::source::git::tag::latest_version(d.path(), "studio").unwrap(),
        Some("1.0.0".into())
    );
}

#[test]
fn test_tags_for_scope() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["cli/v0.1.0", "cli/v0.2.0", "studio/v0.1.0"]);
    let tags = quanttide_devops::source::git::tag::tags_for_scope(d.path(), "cli").unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"cli/v0.1.0".to_string()));
    assert!(tags.contains(&"cli/v0.2.0".to_string()));
}

#[test]
fn test_tags_for_scope_no_match() {
    let d = tempfile::tempdir().unwrap();
    init_repo_with_tags(d.path(), &["v1.0.0"]);
    assert!(
        quanttide_devops::source::git::tag::tags_for_scope(d.path(), "cli")
            .unwrap()
            .is_empty()
    );
}
