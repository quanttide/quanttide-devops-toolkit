//! 演示 `ReleaseStatus` 和 `ReleaseState` 的使用方式。
//!
//! 模拟一个 scope 的发布生命周期：创建 → 提交 → 发布 → 再提交 → 版本冲突。

use quanttide_devops::stage::release::{ReleaseState, ReleaseStatus};

fn main() {
    let mut states: Vec<ReleaseState> = Vec::new();

    // ── 1) 从未发布 ────────────────────────────────────────────────
    states.push(ReleaseState {
        status: ReleaseStatus::Unreleased,
        scope: "cli".into(),
        scope_path: "src/cli".into(),
        current_version: None,
        pending_commits: 0,
        changelog: "CHANGELOG.md".into(),
        version_consistent: None,
    });

    // ── 2) 发布了 v0.1.0，无后续提交 ──────────────────────────────
    states.push(ReleaseState {
        status: ReleaseStatus::Latest,
        scope: "cli".into(),
        scope_path: "src/cli".into(),
        current_version: Some("v0.1.0".into()),
        pending_commits: 0,
        changelog: "CHANGELOG.md".into(),
        version_consistent: Some(true),
    });

    // ── 3) 提了 3 个 commit 待发布 ──────────────────────────────────
    states.push(ReleaseState {
        status: ReleaseStatus::Pending,
        scope: "cli".into(),
        scope_path: "src/cli".into(),
        current_version: Some("v0.1.0".into()),
        pending_commits: 3,
        changelog: "CHANGELOG.md".into(),
        version_consistent: Some(true),
    });

    // ── 4) 发布了 v0.2.0，但配置文件中版本号忘记更新 ────────────────
    states.push(ReleaseState {
        status: ReleaseStatus::Inconsistent,
        scope: "cli".into(),
        scope_path: "src/cli".into(),
        current_version: Some("v0.2.0".into()),
        pending_commits: 1,
        changelog: "CHANGELOG.md".into(),
        version_consistent: Some(false),
    });

    // ── 5) git 命令失败，状态不可知 ─────────────────────────────────
    states.push(ReleaseState {
        status: ReleaseStatus::Unknown,
        scope: "unknown-service".into(),
        scope_path: "apps/unknown-service".into(),
        current_version: None,
        pending_commits: 0,
        changelog: "CHANGELOG.md".into(),
        version_consistent: None,
    });

    // ── 输出 ──────────────────────────────────────────────────────
    println!("发布生命周期演示\n{}", "─".repeat(40));
    for s in &states {
        print!("{}", s);
    }
}
