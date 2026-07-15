//! 场景：CLI 需要解析 ROADMAP.md 展示每个版本的完成进度，并自动发现格式问题（版本号不规范、
//! 分类大小写不标准、checkbox 格式错误）帮助用户整改。
//!
//! 手写 regex 解析 ROADMAP 容易漏边界情况（v 前缀、中文状态标签、未知分类）。
//! `Roadmap::from_str` 按 Keep a Changelog 变体解析，支持自定义分类；`validate` 报告
//! 每行问题位置，便于 CLI 给出精确的修改建议。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example source_roadmap
//! ```

use quanttide_devops::source::roadmap::Roadmap;

fn main() {
    // 模拟一个 monorepo 的 ROADMAP 内容
    let input = "\
# ROADMAP — cli

> 格式说明。

## [0.2.0] — 待实施

### Added
- [ ] 支持 workspace 级别的 tag
- [ ] 自动生成 CHANGELOG

### Fixed
- [ ] 契约 loading 的兜底逻辑

## [0.1.0] — 已发布

### Added
- [x] 契约加载与版本一致性检查
- [x] scope 列表与语言检测
- [x] git tag 的 semver 排序
";

    let roadmap = Roadmap::from_str(input).expect("ROADMAP 解析失败");

    // 1. 版本概览
    println!("版本概览 ({} 个):", roadmap.versions().len());
    for v in roadmap.versions() {
        println!(
            "   [{:>8}] {:6}  {}/{} ({:3.0}%)  {}",
            v.version,
            v.status,
            v.done,
            v.total,
            v.percent(),
            "-".repeat((v.percent() / 10.0) as usize),
        );
    }

    println!(
        "\n全局进度: {}/{} ({:.0}%)",
        roadmap.total_done(),
        roadmap.total_all(),
        if roadmap.total_all() == 0 {
            100.0
        } else {
            roadmap.total_done() as f64 / roadmap.total_all() as f64 * 100.0
        }
    );

    // 2. 各版本分类明细
    println!();
    for v in roadmap.versions() {
        println!("[{}] — {}", v.version, v.status);
        for (cat, items) in &v.categories {
            print!("    {}:", cat);
            for item in items {
                let mark = if item.completed { "[x]" } else { "[ ]" };
                print!(" {} {}", mark, item.description);
            }
            println!();
        }
    }

    // 3. 格式验证
    println!("\n格式验证:");
    let issues = roadmap.validate("cli");
    if issues.is_empty() {
        println!("  无格式问题");
    } else {
        for issue in &issues {
            println!("  行 {:4}: {} — {}", issue.line, issue.scope, issue.message);
        }
    }

    // 4. 故意构造一个有问题的情况演示验证
    println!("\n验证问题演示:");
    let bad_input = "\
# ROADMAP

## [abc] — 待实施

### added
- [X] 大写 X 不是标准 checkbox
";
    let bad_roadmap = Roadmap::from_str(bad_input).unwrap();
    let issues = bad_roadmap.validate("demo");
    for issue in &issues {
        println!("  行 {:4}: {}", issue.line, issue.message);
    }
}
