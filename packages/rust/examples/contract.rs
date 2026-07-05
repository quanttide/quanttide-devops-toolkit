//! 契约加载与版本状态检查 — toolkit 核心 API 演示。
//!
//! 展示如何加载契约配置、自动检测语言和 scope、检查版本一致性。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example contract /path/to/repo
//! ```
//!
//! 不传路径时，默认使用当前目录。

use std::path::PathBuf;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("📦 repository: {}", repo_path.display());
    println!();

    // 1. 加载契约（自动检测兜底）
    let contract = match quanttide_devops::contract::load(&repo_path) {
        Ok(c) => {
            println!("✅ 契约加载成功");
            c
        }
        Err(e) => {
            println!("⚠  契约文件不存在 ({}), 使用自动检测", e);
            auto_detect_contract(&repo_path)
        }
    };

    // 2. scope 列表
    println!();
    println!("📋 scopes ({} 个):", contract.scopes.len());
    for scope in &contract.scopes {
        let lang =
            quanttide_devops::source::config_file::detect_language(&repo_path.join(&scope.dir));
        println!(
            "   {:<16} dir: {:<24} lang: {}",
            scope.name,
            scope.dir,
            lang.as_str()
        );
    }

    // 3. 版本状态
    println!();
    println!("🔖 版本状态:");
    for scope in &contract.scopes {
        let state = quanttide_devops::contract::verify_version(&repo_path, scope);
        match state {
            Ok(vs) => {
                let tag = vs.tag_version.as_deref().unwrap_or("—");
                let cfg = vs.config_version.as_deref().unwrap_or("—");
                let mark = if vs.consistent { "✅" } else { "⚠" };
                println!(
                    "   {} {:<12} tag: {:<12} config: {:<12}",
                    mark, scope.name, tag, cfg
                );
            }
            Err(e) => {
                println!("   ⚠ {}: {}", scope.name, e);
            }
        }
    }
}

/// 无 contract.yaml 时自动推测仓库结构生成契约。
fn auto_detect_contract(repo_path: &std::path::Path) -> quanttide_devops::contract::Contract {
    use quanttide_devops::contract::{self, Contract, Language, Registry, Scope, Stage};

    let root_lang = quanttide_devops::source::config_file::detect_language(repo_path);
    let mut scopes: Vec<Scope> = Vec::new();

    // 扫描常见 scope 子目录
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
                let name = sub
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let sub_lang = quanttide_devops::source::config_file::detect_language(&sub);
                if matches!(sub_lang, Language::Unknown(_)) {
                    continue;
                }
                let name_clone = name.clone();
                scopes.push(Scope {
                    name,
                    dir: format!("{}/{}", base, name_clone),
                    language: sub_lang.clone(),
                    build_tool: infer_build_tool(&sub_lang),
                    framework: String::new(),
                    registry: Registry::Crates,
                    release: contract::StageRelease::default(),
                    test_threshold: None,
                    ci_workflow: None,
                });
            }
        }
    }

    // 根目录 scope
    if !matches!(root_lang, Language::Unknown(_)) {
        scopes.insert(
            0,
            Scope {
                name: "(root)".into(),
                dir: ".".into(),
                language: root_lang.clone(),
                build_tool: infer_build_tool(&root_lang),
                framework: String::new(),
                registry: Registry::Crates,
                release: contract::StageRelease::default(),
                test_threshold: None,
                ci_workflow: None,
            },
        );
    }

    Contract {
        stages: Stage {
            build: contract::StageBuild {
                command: Some("cargo build".into()),
            },
            test: contract::StageTest {
                command: Some("cargo test".into()),
                ..contract::StageTest::default()
            },
            release: contract::StageRelease {
                changelog: "CHANGELOG.md".into(),
                pre_publish: Vec::new(),
            },
        },
        scopes,
        ..Contract::default()
    }
}

fn infer_build_tool(
    lang: &quanttide_devops::contract::Language,
) -> quanttide_devops::contract::BuildTool {
    use quanttide_devops::contract::{BuildTool, Language};
    match lang {
        Language::Rust => BuildTool::Cargo,
        Language::Python => BuildTool::Uv,
        Language::Go => BuildTool::Go,
        Language::Dart => BuildTool::Flutter,
        Language::TypeScript => BuildTool::Npm,
        Language::Unknown(_) => BuildTool::Unknown("auto".into()),
    }
}
