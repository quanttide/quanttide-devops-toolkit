//! CHANGELOG 读取、查询与条目追加 — 与 `changelog` 示例互补（后者侧重生成）。
//!
//! 展示：`from_str` / `from_path` → `contains_version` / `release_notes` / `versions` / `latest_version` → `append_entry`。
//!
//! # 运行
//!
//! ```sh
//! cargo run --example changelog_edit
//! ```

fn main() {
    // ── 1. 从字符串解析 ──────────────────────────────────────────
    let content = "\
# CHANGELOG

## [0.2.0] - 2026-07-15

### Added
- Git tag semver 排序支持。
- 契约版本一致性检查。

## [0.1.0] - 2026-06-01

### Added
- 初始版本：基础契约加载与 scope 列表。
";

    let changelog =
        quanttide_devops::source::changelog::Changelog::from_str(content).expect("解析失败");

    // 版本列表（保持文件中顺序：最新优先）
    println!("1. 版本列表");
    for v in changelog.versions() {
        let has_notes = changelog.release_notes(v).is_some();
        let marker = if has_notes { "✓" } else { "✗" };
        println!("   {} {:<12} latest={}", marker, v, changelog.latest_version() == Some(v));
    }

    // 2. 查询指定版本的 release notes
    println!("\n2. 查询 release notes");
    for ver in &["0.2.0", "0.1.0", "0.3.0"] {
        match changelog.release_notes(ver) {
            Some(notes) => {
                let first_line = notes.lines().next().unwrap_or("");
                println!("   {} → {}", ver, first_line);
            }
            None => println!("   {} → (不存在)", ver),
        }
    }

    // 3. contains_version
    println!("\n3. 版本存在性检查");
    for ver in &["0.2.0", "0.1.0", "0.3.0"] {
        let exists = changelog.contains_version(ver);
        println!("   {} → {}", ver, if exists { "存在" } else { "不存在" });
    }

    // ── 4. append_entry（在临时目录操作） ────────────────────────
    println!("\n4. append_entry — 追加新版本条目");
    let dir = tempfile::tempdir().expect("创建临时目录失败");
    let path = dir.path().join("CHANGELOG.md");

    // 4a. 文件不存在时创建
    let ok = quanttide_devops::source::changelog::append_entry(
        &path,
        "0.1.0",
        "### Added\n- 初始版本。",
    )
    .expect("追加失败");
    println!("   创建新文件: {}", if ok { "是" } else { "否" });
    let raw = std::fs::read_to_string(&path).unwrap();
    println!("   内容预览:\n{}", preview(&raw, 4));

    // 4b. 追加新版本（自动插入到已有版本之前）
    let ok = quanttide_devops::source::changelog::append_entry(
        &path,
        "0.2.0",
        "### Added\n- 新功能。",
    )
    .expect("追加失败");
    println!("   追加 0.2.0: {}", if ok { "成功" } else { "跳过" });
    let raw = std::fs::read_to_string(&path).unwrap();
    println!("   内容预览:\n{}", preview(&raw, 6));
    // 验证顺序：0.2.0 在 0.1.0 之前
    let p1 = raw.find("0.2.0").unwrap();
    let p2 = raw.find("0.1.0").unwrap();
    println!("   顺序: 0.2.0({}) < 0.1.0({})? {}", p1, p2, if p1 < p2 { "✓" } else { "✗" });

    // 4c. 已存在的版本不会重复写入
    let ok = quanttide_devops::source::changelog::append_entry(
        &path,
        "0.1.0",
        "### Added\n- 重复。",
    )
    .expect("追加失败");
    println!("   重复追加 0.1.0: {}", if ok { "写入" } else { "跳过" });

    // 4d. scope 前缀的版本号自动标准化
    let ok = quanttide_devops::source::changelog::append_entry(
        &path,
        "cli/0.3.0",
        "### Added\n- CLI 发布。",
    )
    .expect("追加失败");
    let raw = std::fs::read_to_string(&path).unwrap();
    println!("   带 scope 前缀追加: {}", if ok { "成功" } else { "跳过" });
    println!(
        "   CHANGELOG 中无 scope 前缀: {}",
        if raw.contains("cli/0.3.0") {
            "✗ 包含 scope 前缀"
        } else {
            "✓ 已剥离"
        }
    );

    // ── 5. from_path ──────────────────────────────────────────────
    println!("\n5. Changelog::from_path — 从文件读取");
    let cl = quanttide_devops::source::changelog::Changelog::from_path(&path).expect("读取失败");
    let all_versions = cl.versions();
    println!("   版本数: {}", all_versions.len());
    println!("   最新版本: {:?}", cl.latest_version());
}

/// 预览内容的前 n 行
fn preview(s: &str, n: usize) -> String {
    s.lines()
        .take(n)
        .enumerate()
        .map(|(i, line)| format!("      {:2}: {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}
