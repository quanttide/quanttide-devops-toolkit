//! `TagSource` trait 与 mock 注入 — 演示 `_with` 系列函数如何解耦 I/O 让逻辑可测。
//!
//! 核心思路（见 AGENTS.md）：
//! - "Trait 的价值不是多态，是划定测试边界"
//! - "即使只有 1 个生产实现，trait + mock 也能解锁单元测试覆盖"
//!
//! # 运行
//!
//! ```sh
//! cargo run --example tag_source_trait
//! ```

use quanttide_devops::source::git_tag::{
    latest_tag_with, latest_version_with, tags_for_scope_with, TagSource, TagError,
};

fn main() {
    // ── MockTagSource：不依赖 git 仓库的 TagSource ──────────────────
    // 模拟一个 monorepo 的 tag 集合
    let mock = mock_tags(&[
        "cli/v0.3.0",
        "cli/v0.2.0",
        "cli/v0.1.0",
        "studio/v0.5.0",
        "studio/v0.4.0",
        "v1.0.0",
        "v0.9.0",
    ]);

    // 1. 注入 mock 查询最新 tag
    println!("1. latest_tag_with — 注入 mock 查询最新 tag");
    for scope in &["cli", "studio", "docs"] {
        match latest_tag_with(&mock, scope).unwrap() {
            Some(tag) => println!("   {:<8} → {}", scope, tag),
            None => println!("   {:<8} → (无，兜底也无匹配)", scope),
        }
    }

    // 2. 注入 mock 查最新版本号
    println!("\n2. latest_version_with — 标准化版本号");
    for scope in &["cli", "studio"] {
        if let Some(v) = latest_version_with(&mock, scope).unwrap() {
            println!("   {:<8} → {}", scope, v);
        }
    }

    // 3. 注入 mock 按 scope 过滤
    println!("\n3. tags_for_scope_with — 按 scope 过滤");
    for scope in &["cli", "studio", "docs"] {
        let tags = tags_for_scope_with(&mock, scope).unwrap();
        if tags.is_empty() {
            println!("   {:<8} → (无)", scope);
        } else {
            println!("   {:<8} → {}", scope, tags.join(", "));
        }
    }

    // 4. 模拟真实的 TagSource 对比
    println!("\n4. 如果换成 GixTagSource（真实 git 仓库），调用方式完全一致：");
    println!("     let source = GixTagSource::new(path);");
    println!("     latest_tag_with(&source, scope)");
    println!("     // 返回类型和错误处理与 mock 用例完全相同");

    // 5. 演示 TagSource 的测试价值
    println!("\n5. Mock 能测到哪些真实仓库难以构造的场景：");
    let edge_cases = [
        ("空仓库", vec![]),
        ("只有 unscoped tag", vec!["v1.0.0".into()]),
        ("pre-release 版本", vec!["cli/v2.0.0-alpha".into(), "cli/v1.9.0".into()]),
        ("scope 名含点", vec!["pkg.name/v0.1.0".into()]),
        ("版本号不合法", vec!["cli/not-a-version".into()]),
    ];
    for (desc, tags) in &edge_cases {
        let source = MockTagSource { tags: tags.clone() };
        let latest = latest_version_with(&source, "cli").unwrap();
        println!("   {:<16} tags: {:?} → latest: {:?}", desc, tags, latest);
    }
}

/// 自定义 MockTagSource — 不依赖 git 仓库，直接返回预设的 tag 列表。
struct MockTagSource {
    tags: Vec<String>,
}

impl TagSource for MockTagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagError> {
        Ok(self.tags.clone())
    }
}

/// 辅助：`&[&str]` → `MockTagSource`
fn mock_tags(tags: &[&str]) -> MockTagSource {
    MockTagSource {
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}
