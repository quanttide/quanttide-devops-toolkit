use std::path::Path;

use crate::contract::Scope;
use crate::contract::version::{normalize_version, read_all_config_versions};

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// Git 源操作错误。
#[derive(Debug)]
pub enum GitSourceError {
    /// 仓库打开失败。
    RepoOpen(String),
    /// git2 内部错误。
    Git2(git2::Error),
}

impl std::fmt::Display for GitSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoOpen(p) => write!(f, "无法打开仓库: {}", p),
            Self::Git2(e) => write!(f, "git2 错误: {}", e),
        }
    }
}

impl std::error::Error for GitSourceError {}

impl From<git2::Error> for GitSourceError {
    fn from(e: git2::Error) -> Self {
        Self::Git2(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 版本一致性
// ═══════════════════════════════════════════════════════════════════════

/// 版本一致性检查结果。
#[derive(Debug)]
pub struct VersionStatus {
    /// 最新 git tag 的版本号（已标准化）。
    pub tag_version: Option<String>,
    /// 配置文件中找到的第一个非空版本号。
    pub config_version: Option<String>,
    /// tag 与配置文件版本是否一致。
    pub consistent: bool,
    /// 所有配置文件的版本号明细。(文件名, 版本号)
    pub config_files: Vec<(String, Option<String>)>,
}

// ═══════════════════════════════════════════════════════════════════════
// tag 读取
// ═══════════════════════════════════════════════════════════════════════

/// 获取指定 scope 的最新 tag，标准化后返回。
///
/// scope 匹配规则：
/// - `cli/v0.1.0` → scope `cli` 匹配，返回 `0.1.0`
/// - `v0.1.0`（无前缀）→ 任何 scope 都不匹配，仅在 scope 无专属 tag 时作为兜底
/// - 使用 semver 排序，修复字符串排序 `v10 < v9` 的问题
pub fn latest_tag(repo_path: &Path, scope_name: &str) -> Result<Option<String>, GitSourceError> {
    let tags = all_tags(repo_path)?;
    let prefix = format!("{}/", scope_name);

    let mut scoped: Vec<&str> = Vec::new();
    let mut unscoped: Vec<&str> = Vec::new();
    for tag in &tags {
        if let Some(rest) = tag.strip_prefix(&prefix) {
            if !rest.is_empty() {
                scoped.push(tag);
            }
        } else if !tag.contains('/') {
            unscoped.push(tag);
        }
    }

    scoped.sort_by(|a, b| semver_desc(a, b));
    unscoped.sort_by(|a, b| semver_desc(a, b));

    match scoped.first() {
        Some(t) => Ok(Some(normalize_version(t))),
        None => Ok(unscoped.first().map(|t| normalize_version(t))),
    }
}

/// 获取指定 scope 的所有 tag（原始格式，未标准化）。
pub fn tags_for_scope(repo_path: &Path, scope_name: &str) -> Result<Vec<String>, GitSourceError> {
    let tags = all_tags(repo_path)?;
    let prefix = format!("{}/", scope_name);
    Ok(tags
        .into_iter()
        .filter(|t| t.starts_with(&prefix))
        .collect())
}

/// 读取仓库中所有 tag 名称。
fn all_tags(repo_path: &Path) -> Result<Vec<String>, GitSourceError> {
    let repo = git2::Repository::open(repo_path)
        .map_err(|_| GitSourceError::RepoOpen(repo_path.display().to_string()))?;
    let tag_names = repo.tag_names(None)?;
    Ok(tag_names.iter().flatten().map(String::from).collect())
}

/// 检查 scope 配置文件版本与最新 git tag 是否一致。
pub fn version_status(repo_path: &Path, scope: &Scope) -> Result<VersionStatus, GitSourceError> {
    let tag_version = latest_tag(repo_path, &scope.name)?;
    let scope_dir = repo_path.join(&scope.dir);
    let config_files = read_all_config_versions(&scope_dir);
    let config_version = config_files
        .iter()
        .find(|(_, v)| v.is_some())
        .and_then(|(_, v)| v.clone());

    let consistent = match &tag_version {
        Some(t) => config_files.iter().all(|(_, v)| match v {
            Some(cv) => cv == t,
            None => true,
        }),
        None => config_version.is_none(),
    };

    Ok(VersionStatus {
        tag_version,
        config_version,
        consistent,
        config_files,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// semver 比较（内联，不引入 semver crate）
// ═══════════════════════════════════════════════════════════════════════

fn parse_semver(tag: &str) -> (u64, u64, u64) {
    let after_scope = tag.split('/').next_back().unwrap_or(tag);
    let ver = after_scope.strip_prefix('v').unwrap_or(after_scope);
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() < 3 {
        return (0, 0, 0);
    }
    let major = parts[0].parse().unwrap_or(0);
    let minor = parts[1].parse().unwrap_or(0);
    let patch_str: String = parts[2]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let patch = patch_str.parse().unwrap_or(0);
    (major, minor, patch)
}

fn semver_desc(a: &str, b: &str) -> std::cmp::Ordering {
    let va = parse_semver(a);
    let vb = parse_semver(b);
    vb.cmp(&va) // 降序：v0.2.0 < v0.1.0 → Less → v0.2.0 排在 v0.1.0 前
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_tags(dir: &Path, tags: &[&str]) {
        let repo = git2::Repository::init(dir).unwrap();
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let tree = {
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            repo.find_tree(oid).unwrap()
        };
        let commit = repo
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();
        for tag in tags {
            repo.tag_lightweight(tag, &repo.find_object(commit, None).unwrap(), false)
                .unwrap();
        }
    }

    // ── latest_tag ────────────────────────────────────────────

    #[test]
    fn test_latest_tag_no_tags() {
        let d = tempfile::tempdir().unwrap();
        git2::Repository::init(d.path()).unwrap();
        assert_eq!(latest_tag(d.path(), "cli").unwrap(), None);
    }

    #[test]
    fn test_latest_tag_scoped() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["cli/v0.2.0", "cli/v0.1.0", "v1.0.0"]);
        assert_eq!(latest_tag(d.path(), "cli").unwrap(), Some("0.2.0".into()));
    }

    #[test]
    fn test_latest_tag_unscoped_fallback() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["v1.0.0"]);
        assert_eq!(latest_tag(d.path(), "cli").unwrap(), Some("1.0.0".into()));
    }

    #[test]
    fn test_latest_tag_semver_sort() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["cli/v9.0.0", "cli/v10.0.0"]);
        assert_eq!(latest_tag(d.path(), "cli").unwrap(), Some("10.0.0".into()));
    }

    #[test]
    fn test_latest_tag_multiple_scopes() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["cli/v0.2.0", "studio/v0.3.0", "cli/v0.1.0"]);
        assert_eq!(latest_tag(d.path(), "cli").unwrap(), Some("0.2.0".into()));
        assert_eq!(
            latest_tag(d.path(), "studio").unwrap(),
            Some("0.3.0".into())
        );
    }

    // ── tags_for_scope ─────────────────────────────────────────

    #[test]
    fn test_tags_for_scope() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["cli/v0.1.0", "cli/v0.2.0", "studio/v0.1.0"]);
        let tags = tags_for_scope(d.path(), "cli").unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"cli/v0.1.0".to_string()));
        assert!(tags.contains(&"cli/v0.2.0".to_string()));
    }

    #[test]
    fn test_tags_for_scope_no_match() {
        let d = tempfile::tempdir().unwrap();
        init_repo_with_tags(d.path(), &["v1.0.0"]);
        assert!(tags_for_scope(d.path(), "cli").unwrap().is_empty());
    }

    // ── parse_semver ───────────────────────────────────────────

    #[test]
    fn test_parse_semver_standard() {
        assert_eq!(parse_semver("v1.2.3"), (1, 2, 3));
    }

    #[test]
    fn test_parse_semver_scoped() {
        assert_eq!(parse_semver("cli/v0.5.0"), (0, 5, 0));
    }

    #[test]
    fn test_parse_semver_prerelease() {
        assert_eq!(parse_semver("v1.0.0-rc.1"), (1, 0, 0));
    }

    #[test]
    fn test_parse_semver_no_v() {
        assert_eq!(parse_semver("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn test_parse_semver_invalid() {
        assert_eq!(parse_semver("not-a-version"), (0, 0, 0));
    }
}
