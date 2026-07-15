//! Git tag 过滤与 semver 解析 — 纯函数演示，不需要 git 仓库。
//!
//! 展示：`parse_semver_tag` → `filter_tags_by_scope` → `filter_latest_tag` → `filter_latest_version`。
//! 这些函数是 CLI `release status` / `release publish` 的底层构建块。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example git_tag
//! ```

use quanttide_devops::source::git_tag::{
    filter_latest_tag, filter_latest_version, filter_tags_by_scope, parse_semver_tag,
};

fn main() {
    // 模拟一个 monorepo 项目中多个 scope 的 tag 集合
    let tags = vec![
        "cli/v0.3.0".into(),
        "cli/v0.2.0".into(),
        "cli/v0.1.0".into(),
        "studio/v0.5.0".into(),
        "studio/v0.4.0".into(),
        "v1.0.0".into(),
        "v0.9.0".into(),
    ];

    // 1. parse_semver_tag: 原始 tag → semver::Version
    println!("1. 解析单个 tag:");
    for tag in &["v1.2.3", "cli/v0.5.0", "not-a-version", "v1.0.0-rc.1"] {
        match parse_semver_tag(tag) {
            Some(v) => println!("   {:<20} → {}.{}.{}", tag, v.major, v.minor, v.patch),
            None => println!("   {:<20} → (无效)", tag),
        }
    }

    // 2. filter_tags_by_scope: 过滤出指定 scope 的所有 tag
    println!("\n2. 按 scope 过滤 tag:");
    for scope in &["cli", "studio", "docs"] {
        let matched = filter_tags_by_scope(&tags, scope);
        if matched.is_empty() {
            println!("   {:<8} → (无匹配)", scope);
        } else {
            println!("   {:<8} → {}", scope, matched.join(", "));
        }
    }

    // 3. filter_latest_tag: scope 最新 tag（scoped 优先，unscoped 兜底）
    println!("\n3. 最新 tag（原始格式）:");
    for scope in &["cli", "studio", "docs"] {
        match filter_latest_tag(&tags, scope) {
            Some(t) => println!("   {:<8} → {}", scope, t),
            None => {
                // docs 无 tag，但 unscoped v1.0.0 应作为兜底
                let fallback = filter_latest_tag(&tags, "");
                println!("   {:<8} → 无专属 tag，兜底: {:?}", scope, fallback);
            }
        }
    }

    // 4. filter_latest_version: 标准化版本号
    println!("\n4. 最新版本号（标准化，去 scope/v 前缀）:");
    for scope in &["cli", "studio"] {
        if let Some(v) = filter_latest_version(&tags, scope) {
            println!("   {:<8} → {}", scope, v);
        }
    }

    // 5. 带 pre-release 的 tag
    println!("\n5. Pre-release tag:");
    let tags_with_prerelease = vec![
        "cli/v2.0.0-alpha".to_string(),
        "cli/v2.0.0-beta".to_string(),
        "cli/v1.9.0".to_string(),
    ];
    for tag in &tags_with_prerelease {
        let parsed = parse_semver_tag(tag);
        println!("   {:<22} → {:?}", tag, parsed.map(|v| v.to_string()));
    }
    println!(
        "   最新 stable: {:?}",
        filter_latest_version(&tags_with_prerelease, "cli")
    );
}
