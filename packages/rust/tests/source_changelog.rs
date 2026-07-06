/// 集成测试：Changelog 错误格式化 + Git 日志收集。

use std::path::Path;
use std::process::Command;

// ═══════════════════════════════════════════════════════════════════════
// source::changelog — Display
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_changelog_error_display() {
    use quanttide_devops::source::changelog::ChangelogError;

    let err = ChangelogError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(err.to_string().contains("读取 CHANGELOG 失败"));

    let err = ChangelogError::Parse("syntax error".into());
    assert!(err.to_string().contains("解析 CHANGELOG 失败"));

    let err = ChangelogError::Git("rev-parse failed".into());
    assert!(err.to_string().contains("git 命令失败"));

    let err = ChangelogError::File("permission denied".into());
    assert!(err.to_string().contains("文件写入失败"));
}

// ═══════════════════════════════════════════════════════════════════════
// source::changelog — collect_git_log
// ═══════════════════════════════════════════════════════════════════════

fn init_repo(dir: &Path) {
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn commit(dir: &Path, msg: &str) {
    std::fs::write(dir.join("f"), msg).unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", msg])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn tag(dir: &Path, name: &str) {
    Command::new("git")
        .args(["tag", name])
        .current_dir(dir)
        .output()
        .unwrap();
}

#[test]
fn test_collect_git_log_with_tag() {
    let d = tempfile::tempdir().unwrap();
    let repo = d.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    init_repo(&repo);
    commit(&repo, "first");
    commit(&repo, "second");
    tag(&repo, "v0.1.0");
    commit(&repo, "third");
    commit(&repo, "fourth");

    let log = quanttide_devops::source::changelog::collect_git_log(&repo, Some("v0.1.0")).unwrap();
    assert_eq!(log.lines().count(), 2);
    assert!(log.contains("third"));
    assert!(log.contains("fourth"));
}

#[test]
fn test_collect_git_log_without_tag() {
    let d = tempfile::tempdir().unwrap();
    let repo = d.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    init_repo(&repo);
    commit(&repo, "first");
    commit(&repo, "second");
    commit(&repo, "third");

    let log = quanttide_devops::source::changelog::collect_git_log(&repo, None::<&str>).unwrap();
    assert_eq!(log.lines().count(), 3);
}

#[test]
fn test_collect_git_log_empty_repo() {
    let d = tempfile::tempdir().unwrap();
    let repo = d.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    init_repo(&repo);

    let err = quanttide_devops::source::changelog::collect_git_log(&repo, None::<&str>).unwrap_err();
    assert!(err.to_string().contains("没有新的提交记录") || err.to_string().contains("git 命令失败"));
}

#[test]
fn test_collect_git_log_no_new_commits() {
    let d = tempfile::tempdir().unwrap();
    let repo = d.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    init_repo(&repo);
    commit(&repo, "first");
    tag(&repo, "v0.1.0");

    let err = quanttide_devops::source::changelog::collect_git_log(&repo, Some("v0.1.0")).unwrap_err();
    assert!(err.to_string().contains("没有新的提交记录"));
}

#[test]
fn test_collect_git_log_not_a_repo() {
    let d = tempfile::tempdir().unwrap();

    let err = quanttide_devops::source::changelog::collect_git_log(d.path(), None::<&str>).unwrap_err();
    assert!(err.to_string().contains("git 命令失败"));
}

// ═══════════════════════════════════════════════════════════════════════
// source::changelog — append_entry
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_append_entry_full_rw() {
    use quanttide_devops::source::changelog::append_entry;

    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("CHANGELOG.md");

    // 写入第一个版本
    let ok = append_entry(&path, "0.1.0", "### Added\n- first").unwrap();
    assert!(ok);

    // 写入第二个版本
    let ok = append_entry(&path, "0.2.0", "### Changed\n- second").unwrap();
    assert!(ok);

    // 读取验证顺序：新版本在前
    let raw = std::fs::read_to_string(&path).unwrap();
    let pos1 = raw.find("0.2.0").unwrap();
    let pos2 = raw.find("0.1.0").unwrap();
    assert!(pos1 < pos2, "0.2.0 应在 0.1.0 之前");
    assert!(raw.starts_with("# CHANGELOG"));
}

#[test]
fn test_append_entry_path_not_found_creates() {
    use quanttide_devops::source::changelog::append_entry;

    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("nonexistent/CHANGELOG.md");

    let result = append_entry(&path, "0.1.0", "test");
    // 父目录不存在，预期文件写入失败
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("文件写入失败"));
}
