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
    /// gix 内部错误。
    Gix(String),
}

impl std::fmt::Display for GitSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoOpen(p) => write!(f, "无法打开仓库: {}", p),
            Self::Gix(e) => write!(f, "gix 错误: {}", e),
        }
    }
}

impl std::error::Error for GitSourceError {}

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
    let repo = gix::open(repo_path)
        .map_err(|e| GitSourceError::RepoOpen(format!("{}: {}", repo_path.display(), e)))?;
    let refs = repo
        .references()
        .map_err(|e| GitSourceError::Gix(e.to_string()))?;
    let iter = refs
        .prefixed("refs/tags")
        .map_err(|e| GitSourceError::Gix(e.to_string()))?;
    let tags: Vec<String> = iter
        .filter_map(|r| r.ok())
        .filter_map(|r| {
            let full = r.name().as_bstr().to_string();
            let short = full.strip_prefix("refs/tags/")?;
            Some(short.to_string())
        })
        .collect();
    Ok(tags)
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
// 测试（纯逻辑，无需真实 git 仓库）
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

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
