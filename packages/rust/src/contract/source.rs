use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Sources（事实源维度）──────────────────────────────────────────────

/// 事实源配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Source {
    #[serde(default)]
    pub version: VersionSource,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            version: VersionSource {
                source_type: SourceType::Auto,
                path: None,
            },
        }
    }
}

/// 版本号来源配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VersionSource {
    /// YAML key 为 `type`（Rust 保留字，故字段命名避开）。
    #[serde(default, rename = "type")]
    pub source_type: SourceType,
    #[serde(default)]
    pub path: Option<String>,
}

impl Default for VersionSource {
    fn default() -> Self {
        Self {
            source_type: SourceType::Auto,
            path: None,
        }
    }
}

/// 版本号读取来源。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Cargo,
    Pyproject,
    /// 不从配置文件读版本，只从 git tag 读。
    TagOnly,
    Pubspec,
    #[serde(rename = "package.json")]
    PackageJson,
    /// 自动检测。
    #[default]
    Auto,
}

impl SourceType {
    /// 根据目录下的文件自动检测版本源类型。
    pub fn detect(dir: &Path) -> Self {
        if dir.join("Cargo.toml").exists() {
            Self::Cargo
        } else if dir.join("pyproject.toml").exists() {
            Self::Pyproject
        } else if dir.join("pubspec.yaml").exists() {
            Self::Pubspec
        } else if dir.join("package.json").exists() {
            Self::PackageJson
        } else {
            Self::TagOnly
        }
    }
}
