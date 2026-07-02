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

type VersionExtract = fn(&str) -> Option<String>;

/// 读取目录下所有已知配置文件的版本号。
///
/// ```
/// use std::path::Path;
/// use quanttide_devops::contract::read_all_config_versions;
/// let versions = read_all_config_versions(Path::new("/tmp/nonexistent"));
/// assert!(versions.is_empty());
/// ```
pub fn read_all_config_versions(dir: &Path) -> Vec<(String, Option<String>)> {
    let checks: &[(&str, VersionExtract)] = &[
        ("Cargo.toml", |c| extract_kv_version(c, "version")),
        ("pyproject.toml", |c| extract_kv_version(c, "version")),
        ("package.json", extract_json_version),
        ("pubspec.yaml", |c| extract_kv_yaml_version(c)),
    ];
    checks
        .iter()
        .filter_map(|(name, extract)| {
            let path = dir.join(name);
            if path.exists() {
                let content = std::fs::read_to_string(&path).ok()?;
                Some((name.to_string(), extract(&content)))
            } else {
                None
            }
        })
        .collect()
}

fn extract_kv_version(content: &str, key: &str) -> Option<String> {
    let p = format!("{} = \"", key);
    for line in content.lines() {
        let t = line.trim();
        if let Some(r) = t.strip_prefix(&p)
            && let Some(end) = r.find('"')
        {
            let v = r[..end].to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

fn extract_json_version(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(pos) = line.find(r#""version":"#) {
            let after_key = line[pos + 10..].trim();
            // 跳过开头的引号
            if let Some(start) = after_key.find('"') {
                let after_open = &after_key[start + 1..];
                if let Some(end) = after_open.find('"') {
                    let v = &after_open[..end];
                    if !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_kv_yaml_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(r) = t.strip_prefix("version:") {
            let v = r.trim();
            if !v.is_empty() && !v.starts_with('#') {
                return Some(v.to_string());
            }
        }
    }
    None
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

    // ── 版本提取 ──────────────────────────────────────────────────

    #[test]
    fn test_extract_kv_version() {
        let c = r#"[package]
name = "test"
version = "1.2.3"
"#;
        assert_eq!(extract_kv_version(c, "version"), Some("1.2.3".into()));
    }

    #[test]
    fn test_extract_kv_version_not_found() {
        assert_eq!(extract_kv_version("", "version"), None);
    }

    #[test]
    fn test_extract_json_version() {
        assert_eq!(
            extract_json_version(r#"{"version": "1.0.0"}"#),
            Some("1.0.0".into())
        );
    }

    #[test]
    fn test_extract_json_version_not_found() {
        assert_eq!(extract_json_version("{}"), None);
    }

    #[test]
    fn test_extract_kv_yaml_version() {
        assert_eq!(
            extract_kv_yaml_version("version: 0.2.0"),
            Some("0.2.0".into())
        );
    }

    #[test]
    fn test_extract_kv_yaml_version_commented() {
        assert_eq!(extract_kv_yaml_version("# version: 0.2.0"), None);
    }

    // ── read_all_config_versions ──────────────────────────────────

    #[test]
    fn test_read_all_config_versions_empty_dir() {
        let d = tempfile::tempdir().unwrap();
        assert!(read_all_config_versions(d.path()).is_empty());
    }

    #[test]
    fn test_read_all_config_versions_cargo() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();
        let versions = read_all_config_versions(d.path());
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].1.as_deref(), Some("0.1.0"));
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
