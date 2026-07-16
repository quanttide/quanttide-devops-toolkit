//! 从 Git tag 读取版本号，作为版本号的事实源之一。
//!
//! # 事实源定位
//!
//! 本模块提供的是**事实**（scope 名和版本号当前是什么），不涉及规则判断。
//! 规则的判定在 `contract::version` 中。
//!
//! # 架构
//!
//! 通过 [`TagSource`] trait 将 I/O（读取 tag）与业务逻辑（过滤、排序）分离。
//!
//! # 示例
//!
//! ```ignore
//! use quanttide_devops::source::git::tag::latest_tag;
//! let tag = latest_tag(repo_path, "cli")?;  // "cli/v0.2.0"
//! let ver = quanttide_devops::source::git::tag::latest_version(repo_path, "cli")?;  // "0.2.0"
//! ```

use crate::contract::version::normalize_version;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// Git tag 读取操作错误。
#[derive(Debug)]
pub enum TagError {
    /// 仓库打开失败。
    RepoOpen(String),
    /// gix 内部错误。
    Gix(String),
}

impl std::fmt::Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoOpen(p) => write!(f, "无法打开仓库: {}", p),
            Self::Gix(e) => write!(f, "gix 错误: {}", e),
        }
    }
}

impl std::error::Error for TagError {}

// ═══════════════════════════════════════════════════════════════════════
// TagSource trait
// ═══════════════════════════════════════════════════════════════════════

/// Tag 列表的抽象来源。
pub trait TagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagError>;
}

/// gix 实现的 [`TagSource`]。
pub struct GixTagSource {
    repo_path: PathBuf,
}

impl GixTagSource {
    pub fn new(path: &Path) -> Self {
        Self {
            repo_path: path.to_path_buf(),
        }
    }
}

impl TagSource for GixTagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagError> {
        let repo = gix::open(&self.repo_path)
            .map_err(|e| TagError::RepoOpen(format!("{}: {}", self.repo_path.display(), e)))?;
        let refs = repo
            .references()
            .map_err(|e| TagError::Gix(e.to_string()))?;
        let iter = refs
            .prefixed("refs/tags")
            .map_err(|e| TagError::Gix(e.to_string()))?;
        Ok(iter
            .filter_map(|r| r.ok())
            .filter_map(|r| {
                let full = r.name().as_bstr().to_string();
                let short = full.strip_prefix("refs/tags/")?;
                Some(short.to_string())
            })
            .collect())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 纯逻辑
// ═══════════════════════════════════════════════════════════════════════

/// 从 tag 列表中选出指定 scope 的最新 tag（原始格式，如 `cli/v0.2.0`）。
///
/// scope 匹配规则：
/// - `cli/v0.1.0` → scope `cli` 匹配，返回 `cli/v0.1.0`
/// - `v0.1.0`（无前缀）→ 任何 scope 都不匹配，仅在 scope 无专属 tag 时作为兜底
/// - 使用 semver 排序
///
/// ```
/// use quanttide_devops::source::git::tag::filter_latest_tag;
///
/// let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
/// assert_eq!(filter_latest_tag(&tags, "cli"), Some("cli/v0.2.0".into()));
/// ```
pub fn filter_latest_tag(tags: &[String], scope_name: &str) -> Option<String> {
    let prefix = format!("{}/", scope_name);
    let mut scoped: Vec<&str> = Vec::new();
    let mut unscoped: Vec<&str> = Vec::new();
    for tag in tags {
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
        Some(t) => Some(t.to_string()),
        None => unscoped.first().map(|t| t.to_string()),
    }
}

/// 从 tag 列表中选出指定 scope 的最新版本号（标准化，如 `0.2.0`）。
///
/// 与 [`filter_latest_tag`] 的区别：后者返回原始 tag 名，本函数返回去 scope 去 v 前缀的版本号。
///
/// ```
/// use quanttide_devops::source::git::tag::filter_latest_version;
///
/// let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
/// assert_eq!(filter_latest_version(&tags, "cli"), Some("0.2.0".into()));
/// ```
pub fn filter_latest_version(tags: &[String], scope_name: &str) -> Option<String> {
    filter_latest_tag(tags, scope_name).map(|t| normalize_version(&t))
}

/// 从 tag 列表中过滤出指定 scope 的 tag。
///
/// ```
/// use quanttide_devops::source::git::tag::filter_tags_by_scope;
///
/// let tags = vec!["cli/v0.1.0".into(), "studio/v0.2.0".into()];
/// assert_eq!(filter_tags_by_scope(&tags, "cli"), vec!["cli/v0.1.0"]);
/// ```
pub fn filter_tags_by_scope(tags: &[String], scope_name: &str) -> Vec<String> {
    let prefix = format!("{}/", scope_name);
    tags.iter()
        .filter(|t| t.starts_with(&prefix))
        .cloned()
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// 公开 API
// ═══════════════════════════════════════════════════════════════════════

/// 获取指定 scope 的最新 tag（原始格式，如 `cli/v0.2.0`）。
///
/// 需要标准化版本号时使用 [`latest_version`]。
pub fn latest_tag(repo_path: &Path, scope_name: &str) -> Result<Option<String>, TagError> {
    latest_tag_with(&GixTagSource::new(repo_path), scope_name)
}

/// 获取指定 scope 的最新版本号（标准化，如 `0.2.0`）。
pub fn latest_version(repo_path: &Path, scope_name: &str) -> Result<Option<String>, TagError> {
    latest_version_with(&GixTagSource::new(repo_path), scope_name)
}

/// 获取指定 scope 的所有 tag（原始格式）。
pub fn tags_for_scope(repo_path: &Path, scope_name: &str) -> Result<Vec<String>, TagError> {
    tags_for_scope_with(&GixTagSource::new(repo_path), scope_name)
}

/// 带注入 [`TagSource`] 的 `latest_tag`。
pub fn latest_tag_with(
    source: &impl TagSource,
    scope_name: &str,
) -> Result<Option<String>, TagError> {
    let tags = source.all_tags()?;
    Ok(filter_latest_tag(&tags, scope_name))
}

/// 带注入 [`TagSource`] 的 `latest_version`。
pub fn latest_version_with(
    source: &impl TagSource,
    scope_name: &str,
) -> Result<Option<String>, TagError> {
    let tags = source.all_tags()?;
    Ok(filter_latest_version(&tags, scope_name))
}

/// 带注入 [`TagSource`] 的 `tags_for_scope`。
pub fn tags_for_scope_with(
    source: &impl TagSource,
    scope_name: &str,
) -> Result<Vec<String>, TagError> {
    let tags = source.all_tags()?;
    Ok(filter_tags_by_scope(&tags, scope_name))
}

// ═══════════════════════════════════════════════════════════════════════
// semver 比较
// ═══════════════════════════════════════════════════════════════════════

pub fn parse_semver_tag(tag: &str) -> Option<semver::Version> {
    let after_scope = tag.split('/').next_back().unwrap_or(tag);
    let ver = after_scope
        .strip_prefix('v')
        .or_else(|| after_scope.strip_prefix('V'))
        .unwrap_or(after_scope);
    semver::Version::parse(ver).ok()
}

fn semver_desc(a: &str, b: &str) -> std::cmp::Ordering {
    let va = parse_semver_tag(a);
    let vb = parse_semver_tag(b);
    match (va, vb) {
        (Some(a), Some(b)) => b.cmp(&a),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTagSource {
        tags: Vec<String>,
    }

    impl TagSource for MockTagSource {
        fn all_tags(&self) -> Result<Vec<String>, TagError> {
            Ok(self.tags.clone())
        }
    }

    fn mock(tags: &[&str]) -> MockTagSource {
        MockTagSource {
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── filter_latest_tag (raw) ─────────────────────────────────────

    #[test]
    fn test_filter_latest_tag_raw_scoped() {
        let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
        assert_eq!(
            filter_latest_tag(&tags, "cli"),
            Some("cli/v0.2.0".into())
        );
    }

    #[test]
    fn test_filter_latest_tag_raw_unscoped_fallback() {
        let tags = vec!["v1.0.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("v1.0.0".into()));
    }

    #[test]
    fn test_filter_latest_tag_raw_empty() {
        let tags: Vec<String> = vec![];
        assert_eq!(filter_latest_tag(&tags, "cli"), None);
    }

    // ── filter_latest_version (normalized) ──────────────────────────

    #[test]
    fn test_filter_latest_version_scoped_wins() {
        let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
        assert_eq!(filter_latest_version(&tags, "cli"), Some("0.2.0".into()));
    }

    #[test]
    fn test_filter_latest_version_unscoped_fallback() {
        let tags = vec!["v1.0.0".into()];
        assert_eq!(filter_latest_version(&tags, "cli"), Some("1.0.0".into()));
    }

    #[test]
    fn test_filter_latest_version_empty() {
        let tags: Vec<String> = vec![];
        assert_eq!(filter_latest_version(&tags, "cli"), None);
    }

    #[test]
    fn test_filter_latest_version_semver_sort() {
        let tags = vec!["cli/v9.0.0".into(), "cli/v10.0.0".into()];
        assert_eq!(
            filter_latest_version(&tags, "cli"),
            Some("10.0.0".into())
        );
    }

    #[test]
    fn test_filter_latest_version_scope_no_match() {
        let tags = vec!["other/v0.1.0".into()];
        assert_eq!(filter_latest_version(&tags, "cli"), None);
    }

    #[test]
    fn test_filter_latest_version_excludes_empty_scope() {
        let tags = vec!["cli/".into(), "v1.0.0".into()];
        assert_eq!(filter_latest_version(&tags, "cli"), Some("1.0.0".into()));
    }

    #[test]
    fn test_filter_latest_version_multi_scope() {
        let tags = vec![
            "cli/v0.2.0".into(),
            "studio/v0.3.0".into(),
            "cli/v0.1.0".into(),
        ];
        assert_eq!(
            filter_latest_version(&tags, "cli"),
            Some("0.2.0".into())
        );
        assert_eq!(
            filter_latest_version(&tags, "studio"),
            Some("0.3.0".into())
        );
    }

    // ── latest_tag_with / latest_version_with ───────────────────────

    #[test]
    fn test_latest_tag_with_mock() {
        let source = mock(&["cli/v0.2.0", "v1.0.0"]);
        assert_eq!(
            latest_tag_with(&source, "cli").unwrap(),
            Some("cli/v0.2.0".into())
        );
    }

    #[test]
    fn test_latest_version_with_mock() {
        let source = mock(&["cli/v0.2.0", "v1.0.0"]);
        assert_eq!(
            latest_version_with(&source, "cli").unwrap(),
            Some("0.2.0".into())
        );
    }

    #[test]
    fn test_latest_tag_with_empty() {
        let source = mock(&[]);
        assert_eq!(latest_tag_with(&source, "cli").unwrap(), None);
    }

    #[test]
    fn test_latest_version_with_empty() {
        let source = mock(&[]);
        assert_eq!(latest_version_with(&source, "cli").unwrap(), None);
    }

    #[test]
    fn test_filter_tags_by_scope_matches() {
        let tags = vec!["cli/v0.1.0".into(), "studio/v0.2.0".into()];
        assert_eq!(filter_tags_by_scope(&tags, "cli"), vec!["cli/v0.1.0"]);
    }

    #[test]
    fn test_filter_tags_by_scope_no_match() {
        let tags = vec!["v1.0.0".into()];
        assert!(filter_tags_by_scope(&tags, "cli").is_empty());
    }

    #[test]
    fn test_tags_for_scope_with_mock() {
        let source = mock(&["cli/v0.1.0", "studio/v0.2.0"]);
        assert_eq!(
            tags_for_scope_with(&source, "cli").unwrap(),
            vec!["cli/v0.1.0"]
        );
    }

    #[test]
    fn test_parse_semver_standard() {
        assert_eq!(
            parse_semver_tag("v1.2.3"),
            Some(semver::Version::new(1, 2, 3))
        );
    }
    #[test]
    fn test_parse_semver_scoped() {
        assert_eq!(
            parse_semver_tag("cli/v0.5.0"),
            Some(semver::Version::new(0, 5, 0))
        );
    }
    #[test]
    fn test_parse_semver_prerelease() {
        let v = parse_semver_tag("v1.0.0-rc.1").unwrap();
        assert!(!v.pre.is_empty());
    }
    #[test]
    fn test_parse_semver_no_v() {
        assert_eq!(
            parse_semver_tag("1.2.3"),
            Some(semver::Version::new(1, 2, 3))
        );
    }
    #[test]
    fn test_parse_semver_invalid() {
        assert_eq!(parse_semver_tag("not-a-version"), None);
    }
    #[test]
    fn test_parse_semver_build_metadata() {
        assert!(parse_semver_tag("v1.0.0+build.1").is_some());
    }
    #[test]
    fn test_parse_semver_complex_prerelease() {
        let v = parse_semver_tag("v2.0.0-alpha.1.2").unwrap();
        assert_eq!(v.pre.to_string(), "alpha.1.2");
    }
    #[test]
    fn test_parse_semver_empty_patch() {
        assert_eq!(parse_semver_tag("v1.0"), None);
    }
    #[test]
    fn test_parse_semver_multiple_scopes() {
        assert_eq!(
            parse_semver_tag("parent/cli/v1.2.3"),
            Some(semver::Version::new(1, 2, 3))
        );
    }
    #[test]
    fn test_parse_semver_uppercase() {
        assert_eq!(
            parse_semver_tag("V1.2.3"),
            Some(semver::Version::new(1, 2, 3))
        );
    }
    #[test]
    fn test_parse_semver_scope_with_dot() {
        assert_eq!(
            parse_semver_tag("pkg.name/v1.2.3"),
            Some(semver::Version::new(1, 2, 3))
        );
    }
    #[test]
    fn test_semver_desc_valid_vs_invalid() {
        assert_eq!(
            semver_desc("v1.0.0", "not-a-version"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            semver_desc("not-a-version", "v1.0.0"),
            std::cmp::Ordering::Greater
        );
    }
    #[test]
    fn test_semver_desc_both_invalid() {
        assert_eq!(
            semver_desc("not-a-version", "also-invalid"),
            std::cmp::Ordering::Equal
        );
    }
    #[test]
    fn test_semver_desc_prerelease_vs_release() {
        assert_eq!(
            semver_desc("v1.0.0-alpha", "v1.0.0"),
            std::cmp::Ordering::Greater
        );
    }
}
