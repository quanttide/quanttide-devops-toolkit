//! Git tag 读取与版本一致性检查。
//!
//! 按 scope 前缀从 git tag 中读取版本号，并与配置文件版本对比一致性。
//!
//! # 架构
//!
//! 通过 [`TagSource`] trait 将 I/O（读取 tag 列表）与业务逻辑（过滤、排序、一致性判断）分离：
//!
//! - [`filter_latest_tag`]、[`filter_tags_by_scope`]、[`check_version_consistency`] 是纯函数，
//!   输入输出都是内存数据，可单元测试。
//! - [`GixTagSource`] 是 `TagSource` 的 gix 实现，负责真实仓库读取。
//! - [`latest_tag`]、[`tags_for_scope`] 是便捷函数，内部使用 `GixTagSource`。
//! - [`latest_tag_with`]、[`tags_for_scope_with`] 接受泛型 `TagSource`，可在测试中注入 mock。
//!
//! # 示例
//!
//! ```ignore
//! use quanttide_devops::source::git_tag::latest_tag;
//!
//! let tag = latest_tag(repo_path, "cli")?;
//! println!("latest cli version: {:?}", tag);
//! ```

use std::path::{Path, PathBuf};

use crate::contract::Scope;
use crate::contract::version::{normalize_version, read_all_config_versions};

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// Git 源操作错误。
#[derive(Debug)]
pub enum TagSourceError {
    /// 仓库打开失败。包含路径和错误原因。
    RepoOpen(String),
    /// gix 内部错误。
    Gix(String),
}

impl std::fmt::Display for TagSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoOpen(p) => write!(f, "无法打开仓库: {}", p),
            Self::Gix(e) => write!(f, "gix 错误: {}", e),
        }
    }
}

impl std::error::Error for TagSourceError {}

// ═══════════════════════════════════════════════════════════════════════
// TagSource trait — I/O 边界
// ═══════════════════════════════════════════════════════════════════════

/// Tag 列表的抽象来源。
///
/// 实现者提供 tag 名称列表，消费方（[`filter_latest_tag`] 等）只依赖此 trait，
/// 不依赖真实 git 仓库。
pub trait TagSource {
    /// 返回仓库中所有 tag 的名称列表。
    fn all_tags(&self) -> Result<Vec<String>, TagSourceError>;
}

/// gix 实现的 [`TagSource`]。
///
/// 从 gix 仓库中读取 `refs/tags/` 下的所有引用，去掉前缀后返回。
pub struct GixTagSource {
    repo_path: PathBuf,
}

impl GixTagSource {
    /// 创建一个新的 gix tag 源，指向 `path` 路径的仓库。
    pub fn new(path: &Path) -> Self {
        Self {
            repo_path: path.to_path_buf(),
        }
    }
}

impl TagSource for GixTagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagSourceError> {
        let repo = gix::open(&self.repo_path).map_err(|e| {
            TagSourceError::RepoOpen(format!("{}: {}", self.repo_path.display(), e))
        })?;
        let refs = repo
            .references()
            .map_err(|e| TagSourceError::Gix(e.to_string()))?;
        let iter = refs
            .prefixed("refs/tags")
            .map_err(|e| TagSourceError::Gix(e.to_string()))?;
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
// 纯逻辑（操作 &[String]，可单元测试）
// ═══════════════════════════════════════════════════════════════════════

/// 从 tag 列表中选出指定 scope 的最新版本，标准化后返回。
///
/// scope 匹配规则：
/// - `cli/v0.1.0` → scope `cli` 匹配，返回 `0.1.0`
/// - `v0.1.0`（无前缀）→ 任何 scope 都不匹配，仅在 scope 无专属 tag 时作为兜底
/// - 使用 semver 排序，修复字符串排序 `v10 < v9` 的问题
///
/// # 示例
///
/// ```
/// use quanttide_devops::source::git_tag::filter_latest_tag;
///
/// let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
/// assert_eq!(filter_latest_tag(&tags, "cli"), Some("0.2.0".into()));
///
/// // 无 scope 专属 tag 时兜底到无前缀 tag
/// let tags = vec!["v1.0.0".into()];
/// assert_eq!(filter_latest_tag(&tags, "cli"), Some("1.0.0".into()));
///
/// // 空列表
/// let tags: Vec<String> = vec![];
/// assert_eq!(filter_latest_tag(&tags, "cli"), None);
///
/// // semver 排序
/// let tags = vec!["cli/v9.0.0".into(), "cli/v10.0.0".into()];
/// assert_eq!(filter_latest_tag(&tags, "cli"), Some("10.0.0".into()));
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
        Some(t) => Some(normalize_version(t)),
        None => unscoped.first().map(|t| normalize_version(t)),
    }
}

/// 从 tag 列表中过滤出指定 scope 的 tag。
///
/// # 示例
///
/// ```
/// use quanttide_devops::source::git_tag::filter_tags_by_scope;
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

/// 检查 tag 版本与配置文件版本是否一致。
///
/// # 一致性规则
///
/// - 有 tag：所有有版本号的配置文件必须与 tag 版本一致，无版本号的忽略
/// - 无 tag：所有配置文件必须无版本号
///
/// # 示例
///
/// ```
/// use quanttide_devops::source::git_tag::check_version_consistency;
///
/// // 一致
/// assert!(check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), Some("0.1.0".into()))],
/// ));
///
/// // 不一致
/// assert!(!check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), Some("0.2.0".into()))],
/// ));
///
/// // 有 tag、配置无版本 → 一致（视为手动管理版本）
/// assert!(check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), None)],
/// ));
///
/// // 无 tag、无版本 → 一致
/// assert!(check_version_consistency(None, &[("Cargo.toml".into(), None)]));
/// ```
pub fn check_version_consistency(
    tag_version: Option<&str>,
    config_files: &[(String, Option<String>)],
) -> bool {
    match tag_version {
        Some(t) => config_files.iter().all(|(_, v)| match v {
            Some(cv) => cv == t,
            None => true,
        }),
        None => config_files.iter().all(|(_, v)| v.is_none()),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 版本一致性
// ═══════════════════════════════════════════════════════════════════════

/// Tag 版本与配置文件版本的一致性检查结果。
#[derive(Debug)]
pub struct VersionStatus {
    /// 最新 git tag 的版本号（已标准化，去 `v` 前缀和 scope 前缀）。
    pub tag_version: Option<String>,
    /// 配置文件中找到的第一个非空版本号。
    pub config_version: Option<String>,
    /// tag 与所有配置文件版本是否一致。
    pub consistent: bool,
    /// 所有配置文件的版本号明细。
    pub config_files: Vec<(String, Option<String>)>,
}

// ═══════════════════════════════════════════════════════════════════════
// 公开 API（便捷函数 + 带注入版本）
// ═══════════════════════════════════════════════════════════════════════

/// 获取指定 scope 的最新 tag，标准化后返回。
///
/// 内部使用 [`GixTagSource`]，从真实仓库读取。
///
/// scope 匹配规则见 [`filter_latest_tag`]。
///
/// # 错误
///
/// - 仓库不存在或无法打开 → `TagSourceError::RepoOpen`
/// - gix 读取失败 → `TagSourceError::Gix`
///
/// # 示例
///
/// ```ignore
/// use quanttide_devops::source::git_tag::latest_tag;
/// let tag = latest_tag("/some/repo".as_ref(), "cli")?;
/// ```
pub fn latest_tag(repo_path: &Path, scope_name: &str) -> Result<Option<String>, TagSourceError> {
    latest_tag_with(&GixTagSource::new(repo_path), scope_name)
}

/// 获取指定 scope 的所有 tag（原始格式，未标准化）。
///
/// # 示例
///
/// ```ignore
/// use quanttide_devops::source::git_tag::tags_for_scope;
/// let tags = tags_for_scope("/some/repo".as_ref(), "cli")?;
/// ```
pub fn tags_for_scope(repo_path: &Path, scope_name: &str) -> Result<Vec<String>, TagSourceError> {
    tags_for_scope_with(&GixTagSource::new(repo_path), scope_name)
}

/// 带注入 [`TagSource`] 的 `latest_tag`。
///
/// 可在测试中注入 mock，无需真实 git 仓库。
///
/// # 示例
///
/// ```
/// use quanttide_devops::source::git_tag::{latest_tag_with, TagSource, TagSourceError};
///
/// struct Mock(&'static [&'static str]);
/// impl TagSource for Mock {
///     fn all_tags(&self) -> Result<Vec<String>, TagSourceError> {
///         Ok(self.0.iter().map(|s| s.to_string()).collect())
///     }
/// }
///
/// let source = Mock(&["cli/v0.2.0", "v1.0.0"]);
/// assert_eq!(latest_tag_with(&source, "cli").unwrap(), Some("0.2.0".into()));
/// ```
pub fn latest_tag_with(
    source: &impl TagSource,
    scope_name: &str,
) -> Result<Option<String>, TagSourceError> {
    let tags = source.all_tags()?;
    Ok(filter_latest_tag(&tags, scope_name))
}

/// 带注入 [`TagSource`] 的 `tags_for_scope`。
///
/// # 示例
///
/// ```
/// use quanttide_devops::source::git_tag::{tags_for_scope_with, TagSource, TagSourceError};
///
/// struct Mock(&'static [&'static str]);
/// impl TagSource for Mock {
///     fn all_tags(&self) -> Result<Vec<String>, TagSourceError> {
///         Ok(self.0.iter().map(|s| s.to_string()).collect())
///     }
/// }
///
/// let source = Mock(&["cli/v0.1.0", "studio/v0.2.0"]);
/// assert_eq!(tags_for_scope_with(&source, "cli").unwrap(), vec!["cli/v0.1.0"]);
/// ```
pub fn tags_for_scope_with(
    source: &impl TagSource,
    scope_name: &str,
) -> Result<Vec<String>, TagSourceError> {
    let tags = source.all_tags()?;
    Ok(filter_tags_by_scope(&tags, scope_name))
}

/// 检查 scope 配置文件版本与最新 git tag 是否一致。
///
/// 结合 `latest_tag` 与 `read_all_config_versions` 的结果做对比。
///
/// # 示例
///
/// ```ignore
/// use quanttide_devops::source::git_tag::version_status;
/// use quanttide_devops::contract::Scope;
///
/// let scope = Scope {
///     name: "cli".into(), dir: "src/cli".into(),
///     ..Default::default()
/// };
/// let status = version_status("/some/repo".as_ref(), &scope)?;
/// println!("consistent: {}", status.consistent);
/// ```
pub fn version_status(repo_path: &Path, scope: &Scope) -> Result<VersionStatus, TagSourceError> {
    let tag_version = latest_tag(repo_path, &scope.name)?;
    let scope_dir = repo_path.join(&scope.dir);
    let config_files = read_all_config_versions(&scope_dir);
    let config_version = config_files
        .iter()
        .find(|(_, v)| v.is_some())
        .and_then(|(_, v)| v.clone());
    let consistent = check_version_consistency(tag_version.as_deref(), &config_files);
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
    vb.cmp(&va)
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
        fn all_tags(&self) -> Result<Vec<String>, TagSourceError> {
            Ok(self.tags.clone())
        }
    }

    fn mock(tags: &[&str]) -> MockTagSource {
        MockTagSource {
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_latest_tag_scoped_wins() {
        let tags = vec!["cli/v0.2.0".into(), "cli/v0.1.0".into(), "v1.0.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("0.2.0".into()));
    }

    #[test]
    fn test_latest_tag_unscoped_fallback() {
        let tags = vec!["v1.0.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("1.0.0".into()));
    }

    #[test]
    fn test_latest_tag_empty() {
        let tags: Vec<String> = vec![];
        assert_eq!(filter_latest_tag(&tags, "cli"), None);
    }

    #[test]
    fn test_latest_tag_semver_sort() {
        let tags = vec!["cli/v9.0.0".into(), "cli/v10.0.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("10.0.0".into()));
    }

    #[test]
    fn test_latest_tag_scoped_only() {
        let tags = vec!["cli/v0.2.0".into(), "cli/v0.3.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("0.3.0".into()));
    }

    #[test]
    fn test_latest_tag_scope_no_match() {
        let tags = vec!["other/v0.1.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), None);
    }

    #[test]
    fn test_latest_tag_excludes_empty_scope() {
        let tags = vec!["cli/".into(), "v1.0.0".into()];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("1.0.0".into()));
    }

    #[test]
    fn test_latest_tag_multi_scope() {
        let tags = vec![
            "cli/v0.2.0".into(),
            "studio/v0.3.0".into(),
            "cli/v0.1.0".into(),
        ];
        assert_eq!(filter_latest_tag(&tags, "cli"), Some("0.2.0".into()));
        assert_eq!(filter_latest_tag(&tags, "studio"), Some("0.3.0".into()));
    }

    #[test]
    fn test_latest_tag_with_mock() {
        let source = mock(&["cli/v0.2.0", "v1.0.0"]);
        assert_eq!(
            latest_tag_with(&source, "cli").unwrap(),
            Some("0.2.0".into())
        );
    }

    #[test]
    fn test_latest_tag_with_empty() {
        let source = mock(&[]);
        assert_eq!(latest_tag_with(&source, "cli").unwrap(), None);
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
    fn test_consistency_matches() {
        assert!(check_version_consistency(
            Some("0.1.0"),
            &[("Cargo.toml".into(), Some("0.1.0".into()))]
        ));
    }

    #[test]
    fn test_consistency_mismatch() {
        assert!(!check_version_consistency(
            Some("0.1.0"),
            &[("Cargo.toml".into(), Some("0.2.0".into()))]
        ));
    }

    #[test]
    fn test_consistency_config_no_version() {
        assert!(check_version_consistency(
            Some("0.1.0"),
            &[("Cargo.toml".into(), None)]
        ));
    }

    #[test]
    fn test_consistency_no_tag_no_config() {
        assert!(check_version_consistency(
            None,
            &[("Cargo.toml".into(), None)]
        ));
    }

    #[test]
    fn test_consistency_no_tag_but_config_has_version() {
        assert!(!check_version_consistency(
            None,
            &[("Cargo.toml".into(), Some("0.1.0".into()))]
        ));
    }

    #[test]
    fn test_consistency_multi_file_all_match() {
        let files = vec![
            ("Cargo.toml".into(), Some("0.1.0".into())),
            ("pyproject.toml".into(), Some("0.1.0".into())),
        ];
        assert!(check_version_consistency(Some("0.1.0"), &files));
    }

    #[test]
    fn test_consistency_multi_file_one_mismatch() {
        let files = vec![
            ("Cargo.toml".into(), Some("0.1.0".into())),
            ("pyproject.toml".into(), Some("0.2.0".into())),
        ];
        assert!(!check_version_consistency(Some("0.1.0"), &files));
    }

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
