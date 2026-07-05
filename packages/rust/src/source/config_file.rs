//! 从配置文件读取版本号，作为版本号的事实源之一。
//!
//! 支持以下文件格式：
//!
//! | 文件 | 提取方式 |
//! |------|---------|
//! | `Cargo.toml` | `version = "..."` 键值对 |
//! | `pyproject.toml` | `version = "..."` 键值对 |
//! | `package.json` | `"version": "..."` JSON |
//! | `pubspec.yaml` | `version: ...` YAML |
//!
//! # 示例
//!
//! ```ignore
//! use quanttide_devops::source::config_file::read_config_versions;
//! let versions = read_config_versions(scope_dir);
//! ```

use std::path::Path;

type VersionExtract = fn(&str) -> Option<String>;

/// 读取目录下所有已知配置文件的版本号。
///
/// 返回 `[(文件名, Option<版本号>)]`，文件不存在则跳过。
///
/// ```
/// use std::path::Path;
/// use quanttide_devops::source::config_file::read_config_versions;
/// let versions = read_config_versions(Path::new("/tmp/nonexistent"));
/// assert!(versions.is_empty());
/// ```
pub fn read_config_versions(dir: &Path) -> Vec<(String, Option<String>)> {
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
    let key = r#""version":"#;
    for line in content.lines() {
        if let Some(pos) = line.find(key) {
            let after_key = line[pos + 10..].trim();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_kv_version_found() {
        let c = "[package]\nname = \"test\"\nversion = \"1.2.3\"\n";
        assert_eq!(extract_kv_version(c, "version"), Some("1.2.3".into()));
    }

    #[test]
    fn test_extract_kv_version_not_found() {
        assert_eq!(extract_kv_version("", "version"), None);
    }

    #[test]
    fn test_extract_json_version_found() {
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
    fn test_extract_kv_yaml_version_found() {
        assert_eq!(
            extract_kv_yaml_version("version: 0.2.0"),
            Some("0.2.0".into())
        );
    }

    #[test]
    fn test_extract_kv_yaml_version_commented() {
        assert_eq!(extract_kv_yaml_version("# version: 0.2.0"), None);
    }

    #[test]
    fn test_read_config_versions_empty_dir() {
        let d = tempfile::tempdir().unwrap();
        assert!(read_config_versions(d.path()).is_empty());
    }

    #[test]
    fn test_read_config_versions_cargo() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let versions = read_config_versions(d.path());
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].1.as_deref(), Some("0.1.0"));
    }
}
