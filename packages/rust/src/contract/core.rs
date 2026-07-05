use super::{platform::*, scope::*, source::*, stage::*};
use serde::{Deserialize, Serialize};
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
            Language::Unknown(_) => crate::source::config_file::detect_languages(scope_dir)
                .into_iter()
                .next()
                .unwrap_or(Language::Unknown("无法识别".into())),
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

    /// 根据目录下的配置文件自动推测仓库结构，生成契约。
    ///
    /// 扫描 `src/`、`packages/`、`apps/` 下的子目录，检测每个子目录的编程语言并创建 scope。
    /// 如果根目录也存在已知配置文件，添加一个 `(root)` scope。
    ///
    /// ```
    /// use std::path::Path;
    /// use quanttide_devops::contract::Contract;
    ///
    /// let c = Contract::auto_detect(Path::new("/tmp/nonexistent"));
    /// assert!(c.scopes.is_empty());
    /// ```
    pub fn auto_detect(repo_path: &Path) -> Self {
        let root_langs = crate::source::config_file::detect_languages(repo_path);

        let mut scopes: Vec<Scope> = Vec::new();
        for base in &["src", "packages", "apps"] {
            let base_dir = repo_path.join(base);
            if !base_dir.is_dir() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if !sub.is_dir() {
                        continue;
                    }
                    let name = match sub.file_name() {
                        Some(n) => n.to_string_lossy().to_string(),
                        None => continue,
                    };
                    let sub_langs = crate::source::config_file::detect_languages(&sub);
                    if sub_langs.is_empty() {
                        continue;
                    }
                    // 子目录通常只有一种语言，取第一个
                    let sub_lang = sub_langs.into_iter().next().unwrap();
                    scopes.push(Scope {
                        name,
                        dir: format!("{}/{}", base, &sub.file_name().unwrap().to_string_lossy()),
                        language: sub_lang.clone(),
                        build_tool: sub_lang.default_build_tool(),
                        framework: String::new(),
                        registry: sub_lang.default_registry(),
                        release: StageRelease::default(),
                        test_threshold: None,
                        ci_workflow: None,
                    });
                }
            }
        }

        if let Some(root_lang) = root_langs.into_iter().next() {
            scopes.insert(
                0,
                Scope {
                    name: "(root)".into(),
                    dir: ".".into(),
                    language: root_lang.clone(),
                    build_tool: root_lang.default_build_tool(),
                    framework: String::new(),
                    registry: root_lang.default_registry(),
                    release: StageRelease::default(),
                    test_threshold: None,
                    ci_workflow: None,
                },
            );
        }

        Self {
            stages: Stage {
                build: StageBuild {
                    command: Some("cargo build".into()),
                },
                test: StageTest {
                    command: Some("cargo test".into()),
                    ..StageTest::default()
                },
                release: StageRelease::default(),
            },
            scopes,
            ..Self::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════
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
        assert_eq!(c.stages.test.threshold, 80.0);
        assert_eq!(c.stages.release.changelog, "CHANGELOG.md");
        assert_eq!(c.platform.source_control, SourceControl::Github);
        assert_eq!(c.sources.version.source_type, SourceType::Auto);
        assert!(c.scopes.is_empty());
    }

    #[test]
    fn test_fully_empty_yaml() {
        let c: Contract = serde_yaml::from_str("").unwrap_or_default();
        assert_eq!(c.stages.test.threshold, 80.0);
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
}
