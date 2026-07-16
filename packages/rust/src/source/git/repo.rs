//! Git 仓库检测：判断路径是否为 Git 仓库。

use std::path::Path;

/// 判断路径是否为 git 仓库（存在 `.git` 目录或文件）。
///
/// `.git` 文件表示该目录是一个 git 子模块的工作树。
pub fn is_git_repo(path: &Path) -> bool {
    let git_dir = path.join(".git");
    git_dir.is_dir() || git_dir.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo_with_dot_git_dir() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join(".git")).unwrap();
        assert!(is_git_repo(d.path()));
    }

    #[test]
    fn test_is_git_repo_with_dot_git_file() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join(".git"), "gitdir: ../.git/modules/foo").unwrap();
        assert!(is_git_repo(d.path()));
    }

    #[test]
    fn test_is_git_repo_false() {
        let d = tempfile::tempdir().unwrap();
        assert!(!is_git_repo(d.path()));
    }

    #[test]
    fn test_is_git_repo_empty_dir() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("empty")).unwrap();
        assert!(!is_git_repo(d.path()));
    }

    #[test]
    fn test_is_git_repo_non_existent_path() {
        let d = tempfile::tempdir().unwrap();
        let non_existent = d.path().join("does_not_exist");
        assert!(!is_git_repo(&non_existent));
    }
}
