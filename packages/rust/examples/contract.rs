//! 场景：CLI 需要加载用户的契约配置，但必须容忍项目没有 `.quanttide/devops/contract.yaml`。
//!
//! `load_or_default` 优先读文件，不存在时自动扫描目录推测 scope 和语言。你需要能
//! 从任意来源构建 Contract（从零构建 / YAML 解析 / auto_detect），并验算 scope 目录是否
//! 存在、用便捷访问器查询 scope 级别的发布配置和测试阈值。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example contract                     # 从零构建 + 序列化 + 自动检测（临时目录）
//! cargo run --example contract /path/to/repo       # 加载实际仓库的契约
//! ```

use std::path::PathBuf;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(demo_tempdir);

    // ── A. 加载契约 ────────────────────────────────────────────────
    let contract = quanttide_devops::contract::load_or_default(&repo_path);
    println!(
        "[A] 契约加载完成 — {} 个 scope",
        contract.scopes.len()
    );

    // scope 列表
    println!();
    for scope in &contract.scopes {
        let langs =
            quanttide_devops::source::config_file::detect_languages(&repo_path.join(&scope.dir));
        let lang_label = langs.first().map(|l| l.as_str()).unwrap_or("—");
        println!(
            "    {:<16} dir: {:<24} lang: {:<12} tool: {}",
            scope.name,
            scope.dir,
            lang_label,
            scope.build_tool.as_str(),
        );
    }

    // ── B. 从 YAML string 构建 Contract ────────────────────────────
    println!();
    println!("[B] 从 YAML string 解析 Contract");
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
    ci_workflow: ci.yml
  studio:
    dir: src/studio
    language: python
    build_tool: uv
    registry: pypi
"#;
    let parsed = quanttide_devops::contract::load_from_str(yaml).expect("YAML 解析失败");
    assert_eq!(parsed.scopes.len(), 2);
    assert_eq!(parsed.scopes[0].name, "cli");
    assert_eq!(parsed.scopes[1].name, "studio");
    println!("    ✓ 解析成功");

    // ── C. 便捷访问器 ──────────────────────────────────────────────
    println!();
    println!("[C] 便捷访问器");
    let cli = &parsed.scopes[0];
    println!(
        "    scope_release(cli).changelog       = {}",
        parsed.scope_release(cli).changelog
    );
    println!(
        "    scope_test_threshold(cli)        = {:.0}",
        parsed.scope_test_threshold(cli)
    );
    let resolved = parsed.resolve_language(cli, &repo_path.join(&cli.dir));
    println!("    resolve_language(cli)            = {}", resolved.as_str());
    match parsed.find_scope_by_path(&repo_path, &repo_path.join("src/cli")) {
        Some(s) => println!("    find_scope_by_path(src/cli)      = {}", s.name),
        None => println!("    find_scope_by_path(src/cli)      = (无匹配)"),
    }

    // ── D. Validate ────────────────────────────────────────────────
    println!();
    println!("[D] Contract::validate — 验算 scope 目录");
    let errors = parsed.validate(&repo_path);
    if errors.is_empty() {
        println!("    ✓ 所有 scope 目录存在");
    } else {
        for e in &errors {
            println!("    ✗ {}", e);
        }
    }

    // ── E. 版本状态 ────────────────────────────────────────────────
    println!();
    println!("[E] 版本状态");
    for scope in &contract.scopes {
        match quanttide_devops::contract::verify_version(&repo_path, scope) {
            Ok(vs) => {
                let tag = vs.tag_version.as_deref().unwrap_or("—");
                let cfg = vs.config_version.as_deref().unwrap_or("—");
                let mark = if vs.consistent { "✓" } else { "✗" };
                println!(
                    "    {} {:<12} tag: {:<12} config: {:<12}",
                    mark, scope.name, tag, cfg
                );
            }
            Err(e) => println!("    ✗ {}: {}", scope.name, e),
        }
    }

    // ── F. Auto-detect ─────────────────────────────────────────────
    println!();
    println!("[F] Contract::auto_detect — 自动推测");
    let auto = quanttide_devops::contract::Contract::auto_detect(&repo_path);
    if auto.scopes.is_empty() {
        println!("    未检测到任何 scope");
    } else {
        for scope in &auto.scopes {
            let langs = quanttide_devops::source::config_file::detect_languages(
                &repo_path.join(&scope.dir),
            );
            let lang_label = langs.first().map(|l| l.as_str()).unwrap_or("—");
            println!(
                "    {:<12} dir: {:<24} lang: {:<12} tool: {}",
                scope.name,
                scope.dir,
                lang_label,
                scope.build_tool.as_str(),
            );
        }
    }
}

/// 生成临时目录演示 auto_detect 和契约加载。
fn demo_tempdir() -> PathBuf {
    let dir = tempfile::tempdir().expect("创建临时目录失败");
    let path = dir.path().to_path_buf();

    std::fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("写入 Cargo.toml 失败");

    std::fs::write(
        path.join("pyproject.toml"),
        "[project]\nname = \"demo-py\"\nversion = \"0.1.0\"\n",
    )
    .expect("写入 pyproject.toml 失败");

    std::fs::create_dir_all(path.join("src/cli")).expect("创建 src/cli 失败");
    std::fs::write(path.join("src/cli/Cargo.toml"), "[package]\nname = \"cli\"\nversion = \"0.1.0\"\n")
        .expect("写入 src/cli/Cargo.toml 失败");

    std::fs::create_dir_all(path.join("src/studio")).expect("创建 src/studio 失败");
    std::fs::write(
        path.join("src/studio/pyproject.toml"),
        "[project]\nname = \"studio\"\nversion = \"0.1.0\"\n",
    )
    .expect("写入 src/studio/pyproject.toml 失败");

    std::mem::forget(dir);
    path
}
