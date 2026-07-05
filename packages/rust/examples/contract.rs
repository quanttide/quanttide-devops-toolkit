//! 契约加载与版本状态检查 — toolkit 核心 API 演示。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example contract /path/to/repo
//! ```
//!
//! 不传路径时，默认使用当前目录。

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("📦 repository: {}", repo_path.display());
    println!();

    // 1. 加载契约（contract.yaml → 自动检测兜底）
    let contract = quanttide_devops::contract::load_or_default(&repo_path);
    println!(
        "{} 契约加载成功",
        if contract.scopes.is_empty() {
            "⚠"
        } else {
            "✅"
        }
    );

    // 2. scope 列表
    println!();
    println!("📋 scopes ({} 个):", contract.scopes.len());
    for scope in &contract.scopes {
        let langs =
            quanttide_devops::source::config_file::detect_languages(&repo_path.join(&scope.dir));
        let lang_label = langs.first().map(|l| l.as_str()).unwrap_or("—");
        println!(
            "   {:<16} dir: {:<24} lang: {} / {}",
            scope.name,
            scope.dir,
            lang_label,
            scope.build_tool.as_str(),
        );
    }

    // 3. 版本状态
    println!();
    println!("🔖 版本状态:");
    for scope in &contract.scopes {
        match quanttide_devops::contract::verify_version(&repo_path, scope) {
            Ok(vs) => {
                let tag = vs.tag_version.as_deref().unwrap_or("—");
                let cfg = vs.config_version.as_deref().unwrap_or("—");
                let mark = if vs.consistent { "✅" } else { "⚠" };
                println!(
                    "   {} {:<12} tag: {:<12} config: {:<12}",
                    mark, scope.name, tag, cfg
                );
            }
            Err(e) => println!("   ⚠ {}: {}", scope.name, e),
        }
    }
}
