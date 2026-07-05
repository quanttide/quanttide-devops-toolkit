//! CHANGELOG 生成演示 — toolkit 版本管理 API 使用示例。
//!
//! 展示：收集 git 提交记录 → 获取最新 tag → 生成 CHANGELOG 条目。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example changelog /path/to/repo
//! ```
//!
//! 不传路径时，默认使用当前目录。

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("📦 仓库: {}", repo_path.display());
    println!();

    // 1. 获取最新 tag（使用 toolkit 封装的 gix 实现）
    let latest = match quanttide_devops::source::git_tag::latest_tag(&repo_path, "") {
        Ok(Some(v)) => v,
        Ok(None) => {
            println!("⚠  没有找到 tag，从头开始收集提交记录");
            String::new()
        }
        Err(e) => {
            eprintln!("❌ 读取 tag 失败: {}", e);
            String::new()
        }
    };
    if !latest.is_empty() {
        println!("🔖 最新 tag: v{}", latest);
    }

    // 2. 收集 git 提交记录（上个 tag 到 HEAD）
    let range = if latest.is_empty() {
        "HEAD".to_string()
    } else {
        format!("v{}..HEAD", latest)
    };

    let log = match Command::new("git")
        .args(["log", "--oneline", &range])
        .current_dir(&repo_path)
        .output()
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                println!("📋 没有新的提交记录");
                return;
            }
            s
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            eprintln!("❌ git log 失败: {}", err);
            return;
        }
        Err(e) => {
            eprintln!("❌ git 执行失败: {}（当前目录不是 git 仓库？）", e);
            return;
        }
    };

    println!("\n📋 提交记录 ({} 条):", log.lines().count());
    for line in log.lines().take(5) {
        println!("   {}", line);
    }
    if log.lines().count() > 5 {
        println!("   ... 还有 {} 条", log.lines().count() - 5);
    }

    // 3.生成 CHANGELOG 条目（输出 LLM prompt，不实际调用 LLM）
    let next_version = prompt_next_version(&latest);
    println!("\n📝 要生成的 CHANGELOG 版本: {}", next_version);
    println!("\n--- LLM Prompt ---");
    println!(
        "根据以下 git 提交记录，为版本 {} 生成 CHANGELOG 条目。\n\
         \n\
         要求：\n\
         1. 按 Added / Changed / Fixed / Removed 分类\n\
         2. 同类提交合并为概括性条目，不要逐条罗列\n\
         3. 用中文描述\n\
         4. 每类不超过 5 条\n\
         5. 仅输出内容，不要版本头部和日期\n\
         \n\
         提交记录：\n{}",
        next_version, log
    );
    println!("---\n");

    // 4. 如果能写文件，模拟写入
    let changelog_path = repo_path.join("CHANGELOG.md");
    if !changelog_path.exists() {
        println!(
            "💡 提示: CHANGELOG.md 不存在，创建后可运行 `cargo run --example changelog .` 重新生成"
        );
    } else {
        println!("📄 CHANGELOG.md 已存在，手动将 LLM 输出粘贴到文件中");
    }
}

/// 根据当前版本号提示下一个版本。
fn prompt_next_version(current: &str) -> String {
    if current.is_empty() {
        "0.1.0".to_string()
    } else {
        // 简单的 patch 递增示意
        let parts: Vec<&str> = current.split('.').collect();
        if parts.len() == 3 {
            let patch: u32 = parts[2].parse().unwrap_or(0);
            format!("{}.{}.{}", parts[0], parts[1], patch + 1)
        } else {
            format!("{}.next", current)
        }
    }
}
