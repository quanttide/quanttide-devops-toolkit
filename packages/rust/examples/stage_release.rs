//! 场景：CLI 需要给用户展示一个 scope 当前处于发布生命周期的哪个阶段——从未发布、
//! 已是最新、有变更待发布、还是版本冲突。
//!
//! 这不是简单的"有没有 tag"。你需要综合 git tag 版本、config 文件版本、pending commits
//! 多个事实源才能算出来。`ReleaseState` + `Display` 帮你把计算和输出分离：计算时你操心
//! 数据和规则，输出时直接 `print!("{}", state)`。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example stage_release
//! ```

use quanttide_devops::stage::release::{ReleaseState, ReleaseStatus};
use quanttide_devops::source::git::tag::{latest_tag_with, latest_version_with, TagSource, TagError};

fn main() {
    // ── 1. 用 MockTagSource 模拟一个 monorepo 的 tag 集合 ────────
    // 模拟 3 个 scope：cli（已发布，有未发布提交）、studio（已是最新）、docs（从未发布）
    let mock = MockTagSource::new(&[
        "cli/v0.3.0",
        "cli/v0.2.0",
        "studio/v0.5.0",
    ]);

    // ── 2. 模拟每个 scope 的契约配置（版本号、变更日志路径） ──────
    // 从配置文件读出的版本，用于一致性判断
    let scopes = vec![
        ("cli", "src/cli", "0.3.0", "CHANGELOG.md", 3),      // 有 3 个未发布提交
        ("studio", "src/studio", "0.5.0", "CHANGELOG.md", 0), // 无未发布提交
        ("docs", "docs", "0.1.0", "CHANGELOG.md", 2),         // 有 tag 但未发布
    ];

    // ── 3. 计算每个 scope 的发布状态 ──────────────────────────────
    let mut states: Vec<ReleaseState> = Vec::new();
    for (name, path, config_ver, changelog, pending) in &scopes {
        // 从 mock 获取最新 tag
        let tag_version = latest_version_with(&mock, name).unwrap();
        let raw_tag = latest_tag_with(&mock, name).unwrap();

        // 一致性检查：使用 toolkit 的规则（有 tag 时所有配置文件版本一致，无 tag 时配置文件必须无版本）
        let consistent = if tag_version.as_deref() == Some(config_ver) {
            Some(true)
        } else if tag_version.is_none() && *config_ver == "0.1.0" {
            Some(false) // docs 有配置文件版本但无 tag → 冲突
        } else {
            tag_version.as_ref().map(|_| false)
        };

        // 判断发布状态
        let status = match (tag_version.as_deref(), pending) {
            (None, _) => ReleaseStatus::Unreleased,
            (Some(_), 0) => ReleaseStatus::Latest,
            (Some(_), _) => ReleaseStatus::Pending,
        };

        // scope 冲突模拟：如果 tag 与 config 版本不一致，覆盖为 Inconsistent
        let status = match (status, consistent) {
            (_, Some(false)) => ReleaseStatus::Inconsistent,
            (s, _) => s,
        };

        states.push(ReleaseState {
            status,
            scope: name.to_string(),
            scope_path: path.to_string(),
            current_version: raw_tag,
            pending_commits: *pending,
            changelog: changelog.to_string(),
            version_consistent: consistent,
        });
    }

    // ── 4. 输出 ──────────────────────────────────────────────────
    println!("发布状态报告 ({} scopes)", states.len());
    println!("{}", "─".repeat(40));
    for s in &states {
        print!("{}", s);
    }

    // ── 5. 添加一条版本冲突的演示 ────────────────────────────────
    println!("\n版本冲突示例:");
    let conflict = ReleaseState::new(
        ReleaseStatus::Inconsistent,
        "web",
        "src/web",
        Some("web/v0.1.0".into()),
        0,
        Some(false),
    );
    print!("{}", conflict);
}

// ═══════════════════════════════════════════════════════════════════════
// MockTagSource
// ═══════════════════════════════════════════════════════════════════════

struct MockTagSource {
    tags: Vec<String>,
}

impl MockTagSource {
    fn new(tags: &[&str]) -> Self {
        Self {
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TagSource for MockTagSource {
    fn all_tags(&self) -> Result<Vec<String>, TagError> {
        Ok(self.tags.clone())
    }
}
