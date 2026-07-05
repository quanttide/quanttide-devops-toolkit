use std::path::Path;

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
}
