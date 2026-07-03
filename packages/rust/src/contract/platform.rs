use serde::{Deserialize, Serialize};
use std::fmt;

// ── Platforms（载体维度）──────────────────────────────────────────────

/// 外部治理载体配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Platform {
    #[serde(default)]
    pub source_control: SourceControl,
    #[serde(default)]
    pub pipeline: Pipeline,
    #[serde(default)]
    pub artifact_registry: Registry,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            source_control: SourceControl::Github,
            pipeline: Pipeline::GithubActions,
            artifact_registry: Registry::None,
        }
    }
}

/// 源代码管理平台。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum SourceControl {
    #[default]
    Github,
    Gitlab,
    Gitee,
}

/// Pipeline 平台。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum Pipeline {
    #[default]
    #[serde(rename = "github_actions")]
    GithubActions,
    #[serde(rename = "gitlab_ci")]
    GitlabCi,
    Jenkins,
}

/// 制品库类型。
///
/// 既可用于全局 `Platforms.artifact_registry`，也可用于 scope 级别覆盖。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum Registry {
    Crates,
    #[serde(rename = "pypi")]
    PyPI,
    #[serde(rename = "pubdev")]
    PubDev,
    Npm,
    #[serde(rename = "github_releases")]
    GitHubReleases,
    Docker,
    #[default]
    #[serde(other)]
    None,
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Crates => write!(f, "crates.io"),
            Self::PyPI => write!(f, "PyPI"),
            Self::PubDev => write!(f, "pub.dev"),
            Self::Npm => write!(f, "npm"),
            Self::GitHubReleases => write!(f, "GitHub Releases"),
            Self::Docker => write!(f, "Docker"),
            Self::None => write!(f, "(none)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_display() {
        assert_eq!(Registry::Crates.to_string(), "crates.io");
        assert_eq!(Registry::PyPI.to_string(), "PyPI");
        assert_eq!(Registry::PubDev.to_string(), "pub.dev");
        assert_eq!(Registry::Npm.to_string(), "npm");
        assert_eq!(Registry::GitHubReleases.to_string(), "GitHub Releases");
        assert_eq!(Registry::Docker.to_string(), "Docker");
        assert_eq!(Registry::None.to_string(), "(none)");
    }
}
