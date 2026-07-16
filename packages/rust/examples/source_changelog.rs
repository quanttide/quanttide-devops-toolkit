//! 场景：CLI 需要管理 CHANGELOG.md——给用户显示某个版本的 release notes、追加新版本条目、
//! 或者从 git log 自动生成草稿。
//!
//! `Changelog::from_str` / `from_path` 解析已有文件（兼容 v 前缀），
//! `contains_version` 避免重复追加，`release_notes` 提取单个版本的正文给 GitHub Release 用。
//! `append_entry` 自动插入到已有版本之前、去重、标准化版本号。`collect_git_log` +
//! `build_changelog_prompt` 配合 LLM 生成 CHANGELOG 草稿。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example source_changelog                # 查询 + 编辑（纯函数，无需仓库）
//! cargo run --example source_changelog /path/to/repo  # 额外执行生成 pipeline（需 git 仓库）
//! ```

use std::path::PathBuf;

fn main() {
    // ── A. 解析与查询 ──────────────────────────────────────────
    let content = "\
# CHANGELOG

## [0.2.0] - 2026-07-15

### Added
- Git tag semver 排序支持。
- 契约版本一致性检查。

## [0.1.0] - 2026-06-01

### Added
- 初始版本：基础契约加载与 scope 列表。
";

    let changelog =
        quanttide_devops::source::changelog::Changelog::from_str(content).expect("解析失败");

    println!("[A] 解析与查询\n");
    for v in changelog.versions() {
        let notes = changelog.release_notes(v).unwrap_or("");
        let first = notes.lines().next().unwrap_or("");
        let is_latest = changelog.latest_version() == Some(v);
        println!(
            "    {:8} {}  — {}",
            v,
            if is_latest { "(最新)" } else { "" },
            first,
        );
    }

    // ── B. append_entry ──────────────────────────────────────────
    println!("\n[B] append_entry — 追加新版本");
    let dir = tempfile::tempdir().expect("创建临时目录失败");
    let path = dir.path().join("CHANGELOG.md");

    // 创建新文件
    quanttide_devops::source::changelog::append_entry(&path, "0.1.0", "### Added\n- 初始版本。")
        .expect("追加失败");
    // 插入新版本到已有版本之前
    quanttide_devops::source::changelog::append_entry(&path, "0.2.0", "### Added\n- 新功能。")
        .expect("追加失败");
    // 已存在的版本跳过
    let skipped = quanttide_devops::source::changelog::append_entry(
        &path,
        "0.1.0",
        "### Added\n- 重复。",
    )
    .expect("追加失败");
    // scope 前缀自动标准化
    quanttide_devops::source::changelog::append_entry(&path, "cli/0.3.0", "### Added\n- CLI 发布。")
        .expect("追加失败");

    let raw = std::fs::read_to_string(&path).unwrap();
    for line in raw.lines().take(8) {
        println!("   {}", line);
    }
    println!("    重复跳过: {}    含 scope 前缀: {}", if skipped { "否" } else { "是" }, if raw.contains("cli/0.3.0") { "是" } else { "否" });

    // ── C. 生成 pipeline（需要 git 仓库） ──────────────────────
    let repo_path = std::env::args().nth(1).map(PathBuf::from);
    if let Some(ref path) = repo_path {
        println!("\n[C] 生成 pipeline — {}", path.display());

        let latest =
            match quanttide_devops::source::git::tag::latest_version(path, "") {
                Ok(Some(v)) => v,
                Ok(None) => {
                    println!("    无 tag，从头收集");
                    String::new()
                }
                Err(e) => {
                    println!("    读取 tag 失败: {}", e);
                    String::new()
                }
            };
        if !latest.is_empty() {
            println!("    最新版本: {}", latest);
        }

        let from_tag = if latest.is_empty() {
            None
        } else {
            Some(latest.as_str())
        };
        match quanttide_devops::source::changelog::collect_git_log(path, from_tag) {
            Ok(log) => {
                let next_version = prompt_next_version(&latest);
                println!("    提交记录: {} 条", log.lines().count());
                println!(
                    "    LLM prompt ({} 字):",
                    quanttide_devops::source::changelog::build_changelog_prompt(
                        &log, &next_version
                    )
                    .len()
                );
            }
            Err(e) => println!("    收集日志失败: {}", e),
        }
    } else {
        println!("\n[C] 生成 pipeline — 跳过（未传入 git 仓库路径）");
    }
}

fn prompt_next_version(current: &str) -> String {
    if current.is_empty() {
        "0.1.0".to_string()
    } else {
        let parts: Vec<&str> = current.split('.').collect();
        if parts.len() == 3 {
            let patch: u32 = parts[2].parse().unwrap_or(0);
            format!("{}.{}.{}", parts[0], parts[1], patch + 1)
        } else {
            format!("{}.next", current)
        }
    }
}
