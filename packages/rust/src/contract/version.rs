use std::path::Path;

use crate::contract::Scope;

/// 校验版本号格式。
///
/// 接受以下格式：
/// - `vX.Y.Z` — 标准语义化版本
/// - `vX.Y.Z-prerelease` — 带预发布后缀
/// - `scope/vX.Y.Z` — 带作用域前缀
///
/// ```
/// use quanttide_devops::contract::validate_version;
/// assert!(validate_version("v1.2.3"));
/// assert!(validate_version("cli/v0.5.0-rc.1"));
/// assert!(!validate_version("1.2.3"));        // 缺 v 前缀
/// assert!(!validate_version("v1.2"));          // 缺 patch
/// assert!(!validate_version(""));              // 空
/// ```
pub fn validate_version(version: &str) -> bool {
    if version.is_empty() {
        return false;
    }

    // 处理 scope/vX.Y.Z 格式
    let ver = if let Some(pos) = version.find('/') {
        let scope = &version[..pos];
        // scope 允许字母、数字、下划线、点、连字符
        if scope.is_empty()
            || !scope
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
        {
            return false;
        }
        &version[pos + 1..]
    } else {
        version
    };

    // 必须 v 开头
    let without_v = match ver.strip_prefix('v') {
        Some(v) => v,
        None => return false,
    };

    // 拆 X.Y.Z-prerelease
    let (semver, _prerelease) = if let Some(dash) = without_v.find('-') {
        let sv = &without_v[..dash];
        let pr = &without_v[dash + 1..];
        // prerelease 不能为空或点开头
        if pr.is_empty() || pr.starts_with('.') {
            return false;
        }
        (sv, Some(pr))
    } else {
        (without_v, None)
    };

    // 验证 X.Y.Z
    let parts: Vec<&str> = semver.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// 标准化版本号：去掉 `v` 前缀和 scope 前缀。
///
/// ```
/// use quanttide_devops::contract::normalize_version;
/// assert_eq!(normalize_version("v1.2.3"), "1.2.3");
/// assert_eq!(normalize_version("cli/v0.5.0"), "0.5.0");
/// ```
pub fn normalize_version(version: &str) -> String {
    let after_scope = version.split('/').next_back().unwrap_or(version);
    after_scope
        .strip_prefix('v')
        .unwrap_or(after_scope)
        .to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// 版本一致性规则
// ═══════════════════════════════════════════════════════════════════════

/// 版本信息快照（State）：记录两个事实源的版本信息及一致性结论。
///
/// 调用 `verify_version` 后得到一个快照，`consistent` 字段是核心判断结果。
/// 之所以不叫 `VersionStatus`，是因为结构体包含多个字段（版本信息），
/// 而不只是一个单一布尔值。`consistent` 字段才是真正的状态值。
#[derive(Debug)]
pub struct VersionState {
    /// 最新 git tag 的版本号（已标准化，去 `v` 前缀和 scope 前缀）。
    pub tag_version: Option<String>,
    /// 配置文件中找到的第一个非空版本号。
    pub config_version: Option<String>,
    /// tag 与所有配置文件版本是否一致。
    pub consistent: bool,
    /// 所有配置文件的版本号明细。
    pub config_files: Vec<(String, Option<String>)>,
}

/// 检查 tag 版本与配置文件版本是否一致。
///
/// `config_files` 来自 `source::version::read_config_versions`、
/// `tag_version` 来自 `source::version::latest_tag`。
/// 本函数只做规则判定，不关心数据来源。
///
/// # 一致性规则
///
/// - 有 tag：所有有版本号的配置文件必须与 tag 版本一致，无版本号的忽略
/// - 无 tag：所有配置文件必须无版本号
///
/// # 示例
///
/// ```
/// use quanttide_devops::contract::check_version_consistency;
///
/// assert!(check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), Some("0.1.0".into()))],
/// ));
/// assert!(!check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), Some("0.2.0".into()))],
/// ));
/// assert!(check_version_consistency(
///     Some("0.1.0"),
///     &[("Cargo.toml".into(), None)],
/// ));
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

/// 从 git tag 和配置文件中读取版本号，验证一致性。
///
/// 组合两个事实源（[`source::git_tag::latest_version`] 和
/// [`source::config_file::read_config_versions`]），应用一致性规则，
/// 返回 [`VersionState`]。
pub fn verify_version(
    repo_path: &Path,
    scope: &Scope,
) -> Result<VersionState, Box<dyn std::error::Error>> {
    let tag_version = crate::source::git_tag::latest_version(repo_path, &scope.name)?;
    let scope_dir = repo_path.join(&scope.dir);
    let config_files = crate::source::config_file::read_config_versions(&scope_dir);
    let config_version = config_files
        .iter()
        .find(|(_, v)| v.is_some())
        .and_then(|(_, v)| v.clone());
    let consistent = check_version_consistency(tag_version.as_deref(), &config_files);
    Ok(VersionState {
        tag_version,
        config_version,
        consistent,
        config_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_version ──────────────────────────────────────────

    #[test]
    fn test_validate_version_standard() {
        assert!(validate_version("v1.2.3"));
    }

    #[test]
    fn test_validate_version_prerelease() {
        assert!(validate_version("v1.2.3-rc.1"));
        assert!(validate_version("v1.2.3-alpha"));
    }

    #[test]
    fn test_validate_version_scoped() {
        assert!(validate_version("cli/v1.2.3"));
        assert!(validate_version("pkg.name/v0.1.0"));
    }

    #[test]
    fn test_validate_version_no_v() {
        assert!(!validate_version("1.2.3"));
    }

    #[test]
    fn test_validate_version_incomplete() {
        assert!(!validate_version("v1.2"));
        assert!(!validate_version("v1"));
    }

    #[test]
    fn test_validate_version_empty() {
        assert!(!validate_version(""));
    }

    #[test]
    fn test_validate_version_scope_only() {
        assert!(!validate_version("cli/"));
    }

    #[test]
    fn test_validate_version_invalid_scope_chars() {
        assert!(!validate_version("bad space/v1.2.3"));
        assert!(!validate_version("/v1.2.3"));
    }

    #[test]
    fn test_validate_version_empty_prerelease() {
        assert!(!validate_version("v1.2.3-"));
        assert!(!validate_version("v1.2.3-."));
    }

    // ── normalize_version ─────────────────────────────────────────

    #[test]
    fn test_normalize_version_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_normalize_version_scoped() {
        assert_eq!(normalize_version("cli/v0.5.0"), "0.5.0");
    }

    #[test]
    fn test_normalize_version_no_prefix() {
        assert_eq!(normalize_version("1.2.3"), "1.2.3");
    }

    // ── check_version_consistency ──────────────────────────────────

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
}
