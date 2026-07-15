//! 场景：CLI 需要回答"这个 scope 最新的发布版本是什么"。但 git tag 可能带 scope 前缀
//!（`cli/v0.1.0`）、v 前缀（`v1.0.0`），甚至是非法 semver（ `not-a-version` ）。
//!
//! `filter_latest_tag` 按 scope 过滤 + semver 排序 + unscoped 兜底，帮你安全地定位
//! 最新版本。但真正的架构难点是：怎么让业务逻辑可测？[`TagSource`] trait 把 tag 来源抽象
//! 出来——生产环境用 `GixTagSource`，测试用 mock，核心逻辑（过滤、排序、兜底）在两种
//! 环境下完全一致。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example source_git_tag
//! ```

use quanttide_devops::source::git_tag::{
    filter_latest_tag, filter_latest_version, filter_tags_by_scope, parse_semver_tag,
    latest_tag_with, latest_version_with, tags_for_scope_with, TagSource, TagError,
};

fn main() {
    let tags = vec![
        "cli/v0.3.0".into(),
        "cli/v0.2.0".into(),
        "cli/v0.1.0".into(),
        "studio/v0.5.0".into(),
        "studio/v0.4.0".into(),
        "v1.0.0".into(),
        "v0.9.0".into(),
    ];

    // ── 1. parse_semver_tag ────────────────────────────────────────
    println!("1. parse_semver_tag — 原始 tag → semver::Version");
    for tag in &["v1.2.3", "cli/v0.5.0", "not-a-version", "v1.0.0-rc.1"] {
        match parse_semver_tag(tag) {
            Some(v) => println!("   {:<20} → {}.{}.{}", tag, v.major, v.minor, v.patch),
            None => println!("   {:<20} → (无效)", tag),
        }
    }

    // ── 2. filter_tags_by_scope ────────────────────────────────────
    println!("\n2. filter_tags_by_scope — 按 scope 过滤");
    for scope in &["cli", "studio", "docs"] {
        let matched = filter_tags_by_scope(&tags, scope);
        if matched.is_empty() {
            println!("   {:<8} → (无)", scope);
        } else {
            println!("   {:<8} → {}", scope, matched.join(", "));
        }
    }

    // ── 3. filter_latest_tag / filter_latest_version ───────────────
    println!("\n3. 最新 tag 与版本号");
    for scope in &["cli", "studio", "docs"] {
        let tag = filter_latest_tag(&tags, scope);
        let ver = filter_latest_version(&tags, scope);
        println!("   {:<8} tag: {:20} ver: {:?}", scope, tag.unwrap_or_default(), ver);
    }

    // ── 4. Pre-release ─────────────────────────────────────────────
    println!("\n4. pre-release 排序");
    let prerelease = vec![
        "cli/v2.0.0-alpha".to_string(),
        "cli/v2.0.0-beta".to_string(),
        "cli/v1.9.0".to_string(),
    ];
    for tag in &prerelease {
        println!("   {:<22} → {:?}", tag, parse_semver_tag(tag).map(|v| v.to_string()));
    }

    // ── 5. TagSource trait + mock 注入 ─────────────────────────────
    println!("\n5. latest_tag_with / latest_version_with / tags_for_scope_with");

    // 用 MockTagSource 替代真实 git 仓库（调用方式与 GixTagSource 完全一致）
    let mock = mock_tags(&[
        "cli/v0.3.0",
        "cli/v0.2.0",
        "studio/v0.5.0",
        "v1.0.0",
    ]);

    for scope in &["cli", "studio", "docs"] {
        let tag = latest_tag_with(&mock, scope).unwrap();
        let ver = latest_version_with(&mock, scope).unwrap();
        let filtered = tags_for_scope_with(&mock, scope).unwrap();
        println!(
            "   {:<8} tag: {:20} ver: {:8}  tags: [{}]",
            scope,
            tag.unwrap_or_default(),
            ver.unwrap_or_default(),
            filtered.join(", "),
        );
    }

    // ── 6. mock 能测到的 edge case ────────────────────────────────
    println!("\n6. Mock 覆盖真实仓库难以构造的场景：");
    let cases = [
        ("空仓库", vec![]),
        ("只有 unscoped", vec!["v1.0.0".into()]),
        ("pre-release 版本", vec!["cli/v2.0.0-alpha".into(), "cli/v1.9.0".into()]),
        ("scope 名含点", vec!["pkg.name/v0.1.0".into()]),
        ("版本号不合法", vec!["cli/not-a-version".into()]),
    ];
    for (desc, tags) in &cases {
        let source = mock_tags_str(tags);
        let latest = latest_version_with(&source, "cli").unwrap();
        println!("   {:<16} → {:?}", desc, latest);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MockTagSource
// ═══════════════════════════════════════════════════════════════════════

struct MockTagSource {
    tags: Vec<String>,
}

impl TagSource for MockTagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagError> {
        Ok(self.tags.clone())
    }
}

fn mock_tags(tags: &[&str]) -> MockTagSource {
    MockTagSource {
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

fn mock_tags_str(tags: &[String]) -> MockTagSource {
    MockTagSource {
        tags: tags.to_vec(),
    }
}
