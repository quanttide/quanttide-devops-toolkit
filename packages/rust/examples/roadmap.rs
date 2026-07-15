//! ROADMAP 解析与验证 — 反映 CLI `plan status` / `plan audit` 的底层逻辑。
//!
//! 展示：解析 ROADMAP.md → 版本进度 → 格式验证。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example roadmap
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
