use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

// ── Contract ──────────────────────────────────────────────────────────

/// 完整契约，对应 `.quanttide/devops/contract.yaml`。
///
/// 按四维架构组织：Stage（时序）、Platform（载体）、Source（事实源）、Scope（边界）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    #[serde(default)]
    pub stages: Stage,
    #[serde(default)]
    pub platform: Platform,
    #[serde(default)]
    pub sources: Source,
    #[serde(default, deserialize_with = "deserialize_scopes")]
    pub scopes: Vec<Scope>,
}

// ── Stages（时序维度）────────────────────────────────────────────────

/// 生命周期阶段配置。
///
/// 不规定"怎么做"，只规定"什么时候检查什么"。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stage {
    #[serde(default)]
    pub build: StageBuild,
    #[serde(default)]
    pub test: StageTest,
    #[serde(default)]
    pub release: StageRelease,
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            build: StageBuild { command: None },
            test: StageTest {
                command: None,
                threshold: 70.0,
            },
            release: StageRelease {
                changelog: "CHANGELOG.md".into(),
                pre_publish: Vec::new(),
            },
        }
    }
}

/// 构建阶段。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageBuild {
    #[serde(default)]
    pub command: Option<String>,
}

/// 测试阶段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageTest {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

impl Default for StageTest {
    fn default() -> Self {
        Self {
            command: None,
            threshold: 70.0,
        }
    }
}

const fn default_threshold() -> f64 {
    70.0
}

/// 发布阶段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageRelease {
    #[serde(default = "default_changelog")]
    pub changelog: String,
    #[serde(default)]
    pub pre_publish: Vec<String>,
}

fn default_changelog() -> String {
    "CHANGELOG.md".into()
}

impl Default for StageRelease {
    fn default() -> Self {
        Self {
            changelog: "CHANGELOG.md".into(),
            pre_publish: Vec::new(),
        }
    }
}

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

fn deserialize_scopes<'de, D>(deserializer: D) -> Result<Vec<Scope>, D::Error>
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

// ── 便捷访问器 ────────────────────────────────────────────────────────

impl Contract {
    /// 获取 scope 的发布配置（scope 级覆盖 → 全局默认）。
    pub fn scope_release<'a>(&'a self, scope: &'a Scope) -> &'a StageRelease {
        let has_custom =
            !scope.release.pre_publish.is_empty() || scope.release.changelog != "CHANGELOG.md";
        if has_custom {
            &scope.release
        } else {
            &self.stages.release
        }
    }

    /// 获取 scope 的测试阈值。
    pub fn scope_test_threshold(&self, scope: &Scope) -> f64 {
        scope.test_threshold.unwrap_or(self.stages.test.threshold)
    }

    /// 根据路径查找匹配的 scope（最长前缀匹配）。
    ///
    /// 例如当前在 `src/cli/sub` 时，`cli` scope（dir: `src/cli`）
    /// 比 root scope（dir: `.`）优先级高。
    pub fn find_scope_by_path(&self, current_dir: &Path) -> Option<&Scope> {
        let current_str = current_dir.to_string_lossy();
        self.scopes
            .iter()
            .filter(|s| current_str.starts_with(&s.dir) || s.dir == ".")
            .max_by_key(|s| s.dir.len())
    }

    /// 语言探测：scope 声明了具体语言则返回，否则按目录文件推测。
    pub fn resolve_language(&self, scope: &Scope, scope_dir: &Path) -> Language {
        match &scope.language {
            Language::Unknown(_) => detect_language_by_files(scope_dir),
            lang => lang.clone(),
        }
    }

    /// 验算契约：检查 scope 配置是否合法。
    ///
    /// 返回所有问题的描述列表，空表示合法。
    ///
    /// ```
    /// use std::path::Path;
    /// use quanttide_devops::contract::Contract;
    ///
    /// let c = Contract::default();
    /// let errors = c.validate(Path::new("/tmp/nonexistent"));
    /// assert!(errors.is_empty()); // 空契约→无 scope 可检查
    /// ```
    pub fn validate(&self, repo_path: &Path) -> Vec<String> {
        let mut errors = Vec::new();
        for scope in &self.scopes {
            let dir = repo_path.join(&scope.dir);
            if !dir.exists() {
                errors.push(format!("scope '{}' 目录不存在: {}", scope.name, scope.dir));
            }
        }
        errors
    }
}

/// 根据目录下的标志文件推测编程语言。
pub fn detect_language_by_files(dir: &Path) -> Language {
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

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    fn parse_yaml(s: &str) -> Contract {
        serde_yaml::from_str(s).expect("YAML 应能解析")
    }

    // ── 完整契约 ──────────────────────────────────────────────────

    #[test]
    fn test_full_contract() {
        let yaml = r#"
stages:
  build:
    command: cargo build
  test:
    command: cargo test
    threshold: 80.0
  release:
    changelog: CHANGELOG.md
    pre_publish:
      - cargo publish

platform:
  source_control: github
  pipeline: github_actions
  artifact_registry: crates

sources:
  version:
    type: cargo

scopes:
  cli:
    dir: src/cli
    language: rust
    build_tool: cargo
    registry: crates
    test_threshold: 90.0
  web:
    dir: src/web
    language: typescript
    build_tool: npm
"#;
        let c: Contract = parse_yaml(yaml);
        assert_eq!(c.stages.build.command.as_deref(), Some("cargo build"));
        assert_eq!(c.stages.test.threshold, 80.0);
        assert_eq!(c.stages.test.command.as_deref(), Some("cargo test"));
        assert_eq!(c.stages.release.changelog, "CHANGELOG.md");
        assert_eq!(
            c.stages.release.pre_publish,
            vec!["cargo publish".to_string()]
        );

        assert_eq!(c.platform.source_control, SourceControl::Github);
        assert_eq!(c.platform.pipeline, Pipeline::GithubActions);
        assert_eq!(c.platform.artifact_registry, Registry::Crates);

        assert_eq!(c.sources.version.source_type, SourceType::Cargo);

        assert_eq!(c.scopes.len(), 2);

        let cli = &c.scopes[0];
        assert_eq!(cli.name, "cli");
        assert_eq!(cli.dir, "src/cli");
        assert_eq!(cli.language, Language::Rust);
        assert_eq!(cli.build_tool, BuildTool::Cargo);
        assert_eq!(cli.registry, Registry::Crates);
        assert_eq!(cli.test_threshold, Some(90.0));

        let web = &c.scopes[1];
        assert_eq!(web.name, "web");
        assert_eq!(web.language, Language::TypeScript);
        assert_eq!(web.build_tool, BuildTool::Npm);
    }

    // ── 最小契约（全默认值） ──────────────────────────────────────

    #[test]
    fn test_empty_contract() {
        let yaml = r#"
stages:
scopes:
"#;
        let c: Contract = parse_yaml(yaml);
        assert_eq!(c.stages.build.command, None);
        assert_eq!(c.stages.test.threshold, 70.0);
        assert_eq!(c.stages.release.changelog, "CHANGELOG.md");
        assert_eq!(c.platform.source_control, SourceControl::Github);
        assert_eq!(c.sources.version.source_type, SourceType::Auto);
        assert!(c.scopes.is_empty());
    }

    #[test]
    fn test_fully_empty_yaml() {
        let c: Contract = serde_yaml::from_str("").unwrap_or_default();
        assert_eq!(c.stages.test.threshold, 70.0);
        assert!(c.scopes.is_empty());
    }

    // ── Language 解析 ─────────────────────────────────────────────

    #[test]
    fn test_language_parse() {
        let c: Contract = parse_yaml(
            r#"
scopes:
  a:
    dir: .
    language: rust
  b:
    dir: .
    language: typescript
  c:
    dir: .
    language: ts
  d:
    dir: .
    language: node
  e:
    dir: .
    language: unknown_lang
"#,
        );
        assert_eq!(c.scopes[0].language, Language::Rust);
        assert_eq!(c.scopes[1].language, Language::TypeScript);
        assert_eq!(c.scopes[2].language, Language::TypeScript);
        assert_eq!(c.scopes[3].language, Language::TypeScript);
        assert_eq!(
            c.scopes[4].language,
            Language::Unknown("unknown_lang".into())
        );
    }

    // ── Registry 解析 ─────────────────────────────────────────────

    #[test]
    fn test_registry_parse() {
        let c: Contract = parse_yaml(
            r#"
platform:
  artifact_registry: pypi
scopes:
  s:
    dir: .
    registry: github_releases
"#,
        );
        assert_eq!(c.platform.artifact_registry, Registry::PyPI);
        assert_eq!(c.scopes[0].registry, Registry::GitHubReleases);
    }

    // ── SourceType 解析 ───────────────────────────────────────────

    #[test]
    fn test_source_type() {
        let c: Contract = parse_yaml(
            r#"
sources:
  version:
    type: package.json
"#,
        );
        assert_eq!(c.sources.version.source_type, SourceType::PackageJson);
    }

    // ── 便捷访问器 ────────────────────────────────────────────────

    #[test]
    fn test_scope_release_fallback() {
        let c: Contract = parse_yaml(
            r#"
stages:
  release:
    changelog: CHANGELOG.md
    pre_publish:
      - cargo publish
scopes:
  cli:
    dir: src/cli
    language: rust
"#,
        );
        let cli = &c.scopes[0];
        let rel = c.scope_release(cli);
        assert_eq!(rel.pre_publish, vec!["cargo publish".to_string()]);
    }

    #[test]
    fn test_scope_release_override() {
        let c: Contract = parse_yaml(
            r#"
stages:
  release:
    changelog: CHANGELOG.md
scopes:
  cli:
    dir: src/cli
    language: rust
    release:
      changelog: docs/CHANGELOG.md
"#,
        );
        let cli = &c.scopes[0];
        let rel = c.scope_release(cli);
        assert_eq!(rel.changelog, "docs/CHANGELOG.md");
    }

    #[test]
    fn test_scope_test_threshold() {
        let c: Contract = parse_yaml(
            r#"
stages:
  test:
    threshold: 70.0
scopes:
  a:
    dir: .
  b:
    dir: .
    test_threshold: 90.0
"#,
        );
        assert_eq!(c.scope_test_threshold(&c.scopes[0]), 70.0);
        assert_eq!(c.scope_test_threshold(&c.scopes[1]), 90.0);
    }

    // ── find_scope_by_path ────────────────────────────────────────

    #[test]
    fn test_find_scope_by_path() {
        let c: Contract = parse_yaml(
            r#"
scopes:
  root:
    dir: .
  cli:
    dir: src/cli
  web:
    dir: src/web
"#,
        );
        assert_eq!(
            c.find_scope_by_path(std::path::Path::new("src/cli/sub"))
                .map(|s| s.name.as_str()),
            Some("cli")
        );
        assert_eq!(
            c.find_scope_by_path(std::path::Path::new("src/web"))
                .map(|s| s.name.as_str()),
            Some("web")
        );
        assert_eq!(
            c.find_scope_by_path(std::path::Path::new("unknown"))
                .map(|s| s.name.as_str()),
            Some("root")
        );
    }

    // ── resolve_language ──────────────────────────────────────────

    #[test]
    fn test_resolve_language_declared() {
        let c: Contract = parse_yaml(
            r#"
scopes:
  cli:
    dir: .
    language: rust
"#,
        );
        let lang = c.resolve_language(&c.scopes[0], std::path::Path::new("/tmp"));
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn test_resolve_language_auto() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        let c: Contract = parse_yaml(
            r#"
scopes:
  cli:
    dir: .
"#,
        );
        let lang = c.resolve_language(&c.scopes[0], d.path());
        assert_eq!(lang, Language::Rust);
    }

    // ── detect_language_by_files ──────────────────────────────────

    #[test]
    fn test_detect_by_files() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            detect_language_by_files(d.path()),
            Language::Unknown("无法识别".into())
        );
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::Rust);
        std::fs::write(d.path().join("go.mod"), "").unwrap();
        // Cargo.toml 优先（顺序检测）
        assert_eq!(detect_language_by_files(d.path()), Language::Rust);
    }
}
