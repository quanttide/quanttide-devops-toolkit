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

    // 2. 收集 git 提交记录（使用 toolkit 封装的 collect_git_log）
    let from_tag = if latest.is_empty() { None } else { Some(latest.as_str()) };
    let log = match quanttide_devops::source::changelog::collect_git_log(&repo_path, from_tag) {
        Ok(log) => log,
        Err(e) => {
            println!("📋 {}", e);
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

    // 3. 生成 CHANGELOG prompt（使用 toolkit 封装的 build_changelog_prompt）
    let next_version = prompt_next_version(&latest);
    println!("\n📝 要生成的 CHANGELOG 版本: {}", next_version);
    println!("\n--- LLM Prompt ---");
    println!(
        "{}",
        quanttide_devops::source::changelog::build_changelog_prompt(&log, &next_version)
    );
    println!("---\n");

    // 4. 提示写入
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
        let parts: Vec<&str> = current.split('.').collect();
        if parts.len() == 3 {
            let patch: u32 = parts[2].parse().unwrap_or(0);
            format!("{}.{}.{}", parts[0], parts[1], patch + 1)
        } else {
            format!("{}.next", current)
        }
    }
}
