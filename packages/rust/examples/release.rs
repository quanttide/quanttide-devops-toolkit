//! 演示 `ReleaseStatus` 和 `ReleaseState` 的使用方式。
//!
//! 模拟一个 scope 的发布生命周期：创建 → 提交 → 发布 → 再提交 → 版本冲突。
//! 所有输出均通过 `Display` trait 完成。

use quanttide_devops::stage::release::{ReleaseState, ReleaseStatus};

fn main() {
    let states = vec![
        ReleaseState::new(ReleaseStatus::Unreleased, "cli", "src/cli", None, 0, None),
        ReleaseState::new(
            ReleaseStatus::Latest,
            "cli",
            "src/cli",
            Some("v0.1.0".into()),
            0,
            Some(true),
        ),
        ReleaseState::new(
            ReleaseStatus::Pending,
            "cli",
            "src/cli",
            Some("v0.1.0".into()),
            3,
            Some(true),
        ),
        ReleaseState::new(
            ReleaseStatus::Inconsistent,
            "cli",
            "src/cli",
            Some("v0.2.0".into()),
            1,
            Some(false),
        ),
        ReleaseState::new(
            ReleaseStatus::Unknown,
            "unknown-service",
            "apps/unknown-service",
            None,
            0,
            None,
        ),
    ];

    println!("发布生命周期演示\n{}", "─".repeat(40));
    for s in &states {
        print!("{}", s);
    }
}
