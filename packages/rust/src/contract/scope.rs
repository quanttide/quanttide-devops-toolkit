use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::platform::Registry;
use super::stage::StageRelease;

// ── Scopes（上下文维度）───────────────────────────────────────────────

/// 作用域（上下文维度）。
///
/// 通过 scope 为不同组件挂载不同的 Stage、Platform、Source 组合。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Scope {
    pub name: String,
    pub dir: String,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub framework: String,
    #[serde(default)]
    pub build_tool: BuildTool,
    #[serde(default)]
    pub registry: Registry,
    #[serde(default)]
    pub release: StageRelease,
    #[serde(default)]
    pub test_threshold: Option<f64>,
    #[serde(default)]
    pub ci_workflow: Option<String>,
}

/// 编程语言。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    Python,
    Go,
    Dart,
    #[serde(rename = "typescript")]
    TypeScript,
    Unknown(String),
}

impl Default for Language {
    fn default() -> Self {
        Self::Unknown("auto".into())
    }
}

impl Language {
    /// 返回语言的显示名称。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Go => "go",
            Self::Dart => "dart",
            Self::TypeScript => "typescript",
            Self::Unknown(s) => s,
        }
    }

    /// 返回该语言最常用的构建工具。
    ///
    /// ```
    /// use quanttide_devops::contract::{BuildTool, Language};
    /// assert_eq!(Language::Rust.default_build_tool(), BuildTool::Cargo);
    /// assert_eq!(Language::Python.default_build_tool(), BuildTool::Uv);
    /// assert_eq!(Language::Go.default_build_tool(), BuildTool::Go);
    /// assert_eq!(Language::Dart.default_build_tool(), BuildTool::Flutter);
    /// assert_eq!(Language::TypeScript.default_build_tool(), BuildTool::Npm);
    /// assert!(matches!(Language::Unknown("x".into()).default_build_tool(), BuildTool::Unknown(_)));
    /// ```
    pub fn default_build_tool(&self) -> BuildTool {
        match self {
            Self::Rust => BuildTool::Cargo,
            Self::Python => BuildTool::Uv,
            Self::Go => BuildTool::Go,
            Self::Dart => BuildTool::Flutter,
            Self::TypeScript => BuildTool::Npm,
            Self::Unknown(_) => BuildTool::Unknown("auto".into()),
        }
    }

    /// 返回该语言最常用的包注册中心。
    ///
    /// ```
    /// use quanttide_devops::contract::{Language, Registry};
    /// assert_eq!(Language::Rust.default_registry(), Registry::Crates);
    /// assert_eq!(Language::Python.default_registry(), Registry::PyPI);
    /// assert_eq!(Language::Dart.default_registry(), Registry::PubDev);
    /// assert_eq!(Language::TypeScript.default_registry(), Registry::Npm);
    /// assert_eq!(Language::Unknown("x".into()).default_registry(), Registry::None);
    /// ```
    pub fn default_registry(&self) -> Registry {
        match self {
            Self::Rust => Registry::Crates,
            Self::Python => Registry::PyPI,
            Self::Go => Registry::GitHubReleases,
            Self::Dart => Registry::PubDev,
            Self::TypeScript => Registry::Npm,
            Self::Unknown(_) => Registry::None,
        }
    }
}

/// 构建工具。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildTool {
    Cargo,
    Uv,
    Go,
    Flutter,
    Npm,
    Unknown(String),
}

impl Default for BuildTool {
    fn default() -> Self {
        Self::Unknown("auto".into())
    }
}

impl BuildTool {
    /// 返回构建工具的显示名称。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cargo => "cargo",
            Self::Uv => "uv",
            Self::Go => "go",
            Self::Flutter => "flutter",
            Self::Npm => "npm",
            Self::Unknown(s) => s,
        }
    }
}

// ── 自定义反序列化（Language / BuildTool）────────────────────────────

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "rust" => Language::Rust,
            "python" => Language::Python,
            "go" => Language::Go,
            "dart" => Language::Dart,
            "typescript" | "ts" | "node" => Language::TypeScript,
            other => Language::Unknown(other.to_string()),
        })
    }
}

impl<'de> Deserialize<'de> for BuildTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "cargo" => BuildTool::Cargo,
            "uv" | "poetry" | "pdm" => BuildTool::Uv,
            "go" => BuildTool::Go,
            "flutter" => BuildTool::Flutter,
            "npm" | "pnpm" | "yarn" | "bun" => BuildTool::Npm,
            other => BuildTool::Unknown(other.to_string()),
        })
    }
}

// ── 自定义反序列化（scopes: map → Vec<Scope>）────────────────────────

/// YAML 中的 scope 原始配置（map 格式的中间表示）。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ScopeConfig {
    dir: String,
    #[serde(default)]
    language: Option<Language>,
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    build_tool: Option<BuildTool>,
    #[serde(default)]
    registry: Option<Registry>,
    #[serde(default)]
    release: Option<StageRelease>,
    #[serde(default)]
    test_threshold: Option<f64>,
    #[serde(default)]
    ci_workflow: Option<String>,
}

pub fn deserialize_scopes<'de, D>(deserializer: D) -> Result<Vec<Scope>, D::Error>
where
    D: Deserializer<'de>,
{
    /// 访问器：将 `{ name: { dir, ... } }` 转为 `[Scope, ...]`。
    struct ScopesVisitor;

    impl<'de> Visitor<'de> for ScopesVisitor {
        type Value = Vec<Scope>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("作用域映射")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut scopes = Vec::new();
            while let Some((name, config)) = access.next_entry::<String, ScopeConfig>()? {
                scopes.push(Scope {
                    name,
                    dir: config.dir,
                    language: config.language.unwrap_or(Language::Unknown("auto".into())),
                    framework: config.framework.unwrap_or_default(),
                    build_tool: config
                        .build_tool
                        .unwrap_or(BuildTool::Unknown("auto".into())),
                    registry: config.registry.unwrap_or(Registry::None),
                    release: config.release.unwrap_or_default(),
                    test_threshold: config.test_threshold,
                    ci_workflow: config.ci_workflow,
                });
            }
            Ok(scopes)
        }
    }

    deserializer.deserialize_map(ScopesVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::Dart.as_str(), "dart");
        assert_eq!(Language::TypeScript.as_str(), "typescript");
        assert_eq!(Language::Unknown("zig".into()).as_str(), "zig");
    }

    #[test]
    fn test_build_tool_as_str() {
        assert_eq!(BuildTool::Cargo.as_str(), "cargo");
        assert_eq!(BuildTool::Uv.as_str(), "uv");
        assert_eq!(BuildTool::Go.as_str(), "go");
        assert_eq!(BuildTool::Flutter.as_str(), "flutter");
        assert_eq!(BuildTool::Npm.as_str(), "npm");
        assert_eq!(BuildTool::Unknown("make".into()).as_str(), "make");
    }

    #[test]
    fn test_language_deserialize() {
        let lang: Language = serde_yaml::from_str("rust").unwrap();
        assert_eq!(lang, Language::Rust);
        let lang: Language = serde_yaml::from_str("zig").unwrap();
        assert_eq!(lang, Language::Unknown("zig".into()));
    }

    #[test]
    fn test_build_tool_deserialize() {
        let tool: BuildTool = serde_yaml::from_str("cargo").unwrap();
        assert_eq!(tool, BuildTool::Cargo);
        let tool: BuildTool = serde_yaml::from_str("make").unwrap();
        assert_eq!(tool, BuildTool::Unknown("make".into()));
    }

    #[test]
    fn test_language_default() {
        assert_eq!(Language::default(), Language::Unknown("auto".into()));
    }

    #[test]
    fn test_build_tool_default() {
        assert_eq!(BuildTool::default(), BuildTool::Unknown("auto".into()));
    }
}
