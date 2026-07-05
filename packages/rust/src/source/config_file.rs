//! 从配置文件读取事实数据（版本号、语言等）。
//!
//! # 版本号
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
//! # 语言检测
//!
//! 根据目录下的标志文件推测编程语言，是 `contract::Contract::resolve_language`
//! 在 scope 未声明语言时的兜底方案。
//!
//! # 示例
//!
//! ```ignore
//! use quanttide_devops::source::config_file::read_config_versions;
//! let versions = read_config_versions(scope_dir);
//! ```

use crate::contract::Language;

use std::path::Path;

type VersionExtract = fn(&str) -> Option<String>;

// ═══════════════════════════════════════════════════════════════════════
// 语言检测
// ═══════════════════════════════════════════════════════════════════════

/// 根据目录下的标志文件推测编程语言（按优先级返回首个匹配）。
///
/// 优先级：Cargo.toml > pyproject.toml/requirements.txt > go.mod > pubspec.yaml > package.json
///
/// ⚠ **已废弃**：请使用 [`detect_languages`] 替代，它会独立检测所有语言，
/// 不丢失 monorepo 多语言信息。
///
/// ```
/// use std::path::Path;
/// use quanttide_devops::source::config_file::detect_language;
///
/// let lang = detect_language(Path::new("/tmp/nonexistent"));
/// assert!(matches!(lang, quanttide_devops::contract::Language::Unknown(_)));
/// ```
#[deprecated(note = "请使用 detect_languages，它独立检测所有语言而非按优先级返回首个")]
pub fn detect_language(dir: &Path) -> Language {
    if dir.join("Cargo.toml").exists() {
        Language::Rust
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        Language::Python
    } else if dir.join("go.mod").exists() {
        Language::Go
    } else if dir.join("pubspec.yaml").exists() {
        Language::Dart
    } else if dir.join("package.json").exists() {
        Language::TypeScript
    } else {
        Language::Unknown("无法识别".into())
    }
}

/// 独立检测目录下的所有编程语言，不依赖优先级（每个标志文件独立检查）。
///
/// monorepo 根目录可能同时存在多种语言的配置文件（如 `Cargo.toml` + `pyproject.toml`），
/// 此函数返回所有匹配的语言，适合需要检查所有工具链的场景（如 CLI `doctor status`）。
///
/// ```
/// use std::path::Path;
/// use quanttide_devops::source::config_file::detect_languages;
///
/// let langs = detect_languages(Path::new("/tmp/nonexistent"));
/// assert!(langs.is_empty());
/// ```
pub fn detect_languages(dir: &Path) -> Vec<Language> {
    let mut result = Vec::new();
    if dir.join("Cargo.toml").exists() {
        result.push(Language::Rust);
    }
    if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        if !result.contains(&Language::Python) {
            result.push(Language::Python);
        }
    }
    if dir.join("go.mod").exists() {
        result.push(Language::Go);
    }
    if dir.join("pubspec.yaml").exists() {
        result.push(Language::Dart);
    }
    if dir.join("package.json").exists() {
        result.push(Language::TypeScript);
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════
// 版本号读取
// ═══════════════════════════════════════════════════════════════════════

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

    // ── 语言检测 ──────────────────────────────────────────────

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_rust() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Rust);
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_unknown() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(detect_language(d.path()), Language::Unknown(_)));
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_python() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Python);
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_python_requirements() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("requirements.txt"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Python);
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_go() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("go.mod"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Go);
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_dart() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Dart);
    }

    #[allow(deprecated)]
    #[test]
    fn test_detect_language_typescript() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("package.json"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::TypeScript);
    }

    // ── 多语言检测 ──────────────────────────────────────────

    #[test]
    fn test_detect_languages_single() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_languages(d.path()), vec![Language::Rust]);
    }

    #[test]
    fn test_detect_languages_multi() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
        let langs = detect_languages(d.path());
        assert!(langs.contains(&Language::Rust));
        assert!(langs.contains(&Language::Python));
        assert_eq!(langs.len(), 2);
    }

    #[test]
    fn test_detect_languages_empty() {
        let d = tempfile::tempdir().unwrap();
        assert!(detect_languages(d.path()).is_empty());
    }

    #[test]
    fn test_detect_languages_python_dedup() {
        // pyproject.toml + requirements.txt 只应产生一个 Python
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
        std::fs::write(d.path().join("requirements.txt"), "").unwrap();
        assert_eq!(detect_languages(d.path()), vec![Language::Python]);
    }

    // ── 版本号提取 ────────────────────────────────────────────

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
