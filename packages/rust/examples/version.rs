//! 版本号验证、标准化与一致性检查 — CLI `release audit` / `release publish` 的核心规则。
//!
//! 展示：`validate_version` → `normalize_version` → `check_version_consistency` → `verify_version`（需 git repo）。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example version                      # 纯函数部分
//! cargo run --example version /path/to/repo        # 加上 verify_version（需 git repo + tag）
//! ```

use std::path::PathBuf;

fn main() {
    let repo_path = std::env::args().nth(1).map(PathBuf::from);
    let _has_repo = repo_path.as_ref().is_some_and(|p| p.join(".git").exists());

    // 1. validate_version: 接受/拒绝哪些格式
    println!("1. validate_version — 版本号格式校验");
    let cases = [
        ("v1.2.3", true),
        ("v0.0.1", true),
        ("v1.2.3-rc.1", true),
        ("v1.2.3-alpha+build", true),
        ("cli/v1.2.3", true),
        ("pkg.name/v0.1.0", true),
        ("1.2.3", false),
        ("v1.2", false),
        ("v1", false),
        ("cli/", false),
        ("bad space/v1.2.3", false),
        ("", false),
        ("v1.2.3-", false),
    ];
    for (input, expected) in &cases {
        let ok = quanttide_devops::contract::validate_version(input);
        let mark = if ok == *expected { "✓" } else { "✗" };
        println!(
            "   {} validate_version({:<24}) → {} (expect {})",
            mark, input, ok, expected
        );
    }

    // 2. normalize_version: 标准化
    println!("\n2. normalize_version — 标准化（去 scope/v 前缀）");
    let cases = [
        ("v1.2.3", "1.2.3"),
        ("cli/v0.5.0", "0.5.0"),
        ("1.2.3", "1.2.3"),
        ("cli/v0.5.0-rc.1", "0.5.0-rc.1"),
    ];
    for (input, expected) in &cases {
        let got = quanttide_devops::contract::normalize_version(input);
        let mark = if got == *expected { "✓" } else { "✗" };
        println!("   {} normalize_version({:<24}) → \"{}\"", mark, input, got);
    }

    // 3. check_version_consistency: 一致性规则
    println!("\n3. check_version_consistency — tag 与配置文件版本一致性");
    let scenarios = [
        ("tag=0.1.0,  Cargo.toml=0.1.0", true),
        ("tag=0.1.0,  Cargo.toml=0.2.0", false),
        ("tag=0.1.0,  Cargo.toml=(无版本)", true),
        ("tag=(无),   Cargo.toml=(无版本)", true),
        ("tag=(无),   Cargo.toml=0.1.0", false),
        (
            "tag=0.1.0,  Cargo.toml=0.1.0, pyproject.toml=0.1.0",
            true,
        ),
        (
            "tag=0.1.0,  Cargo.toml=0.1.0, pyproject.toml=0.2.0",
            false,
        ),
    ];
    for (desc, expected) in &scenarios {
        let (tag, files) = parse_scenario_to_owned(desc);
        let ok = quanttide_devops::contract::check_version_consistency(tag, &files);
        let mark = if ok == *expected { "✓" } else { "✗" };
        let status = if ok { "一致" } else { "冲突" };
        println!("   {} {:<55} → {}", mark, desc, status);
    }

    // 4. verify_version: 实际仓库扫描（需要 git repo + 目录下有配置文件）
    if let Some(ref path) = repo_path {
        println!("\n4. verify_version — 在仓库中实际验证");
        let contract = quanttide_devops::contract::load_or_default(path);
        for scope in &contract.scopes {
            match quanttide_devops::contract::verify_version(path, scope) {
                Ok(vs) => {
                    let tag = vs.tag_version.as_deref().unwrap_or("(无)");
                    let cfg = vs.config_version.as_deref().unwrap_or("(无)");
                    let mark = if vs.consistent { "✓" } else { "✗" };
                    println!("   {} {}", mark, scope.name);
                    println!("      tag 版本:        {}", tag);
                    println!("      config 版本:     {}", cfg);
                    for (file, ver) in &vs.config_files {
                        let v = ver.as_deref().unwrap_or("(无版本)");
                        println!("         {}: {}", file, v);
                    }
                }
                Err(e) => println!("   ✗ {}: {}", scope.name, e),
            }
        }
    } else {
        println!("\n4. verify_version — 跳过（未传入 git 仓库路径）");
        println!("   传入路径以验证: cargo run --example version /path/to/repo");
    }
}

/// 解析场景描述 → (tag, config_files)。
fn parse_scenario_to_owned(s: &str) -> (Option<&str>, Vec<(String, Option<String>)>) {
    let mut tag: Option<&str> = None;
    let mut files = Vec::new();

    for part in s.split(',').map(|p| p.trim()) {
        if let Some(v) = part.strip_prefix("tag=") {
            tag = if v == "(无)" { None } else { Some(v) };
        } else if let Some(file_val) = part.split_once('=') {
            let file = file_val.0.trim();
            let val = if file_val.1.trim() == "(无版本)" {
                None
            } else {
                Some(file_val.1.trim().to_string())
            };
            files.push((file.to_string(), val));
        }
    }

    (tag, files)
}
