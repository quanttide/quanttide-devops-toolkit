//! 配置文件版本读取与语言检测 — 反映 CLI `contract status` / `build status` 的底层逻辑。
//!
//! 展示：`detect_languages` → `read_config_versions` → `Contract::auto_detect`。
//!
//! # 运行
//!
//! ```bash
//! cargo run --example config_file /path/to/repo
//! ```
//!
//! 不传路径时，在当前目录生成临时文件来演示。

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(demo_tempdir);

    println!("目标目录: {}", repo_path.display());
    println!();

    // 1. 语言检测
    let langs = quanttide_devops::source::config_file::detect_languages(&repo_path);
    if langs.is_empty() {
        println!("语言检测: (无匹配)");
    } else {
        let labels: Vec<&str> = langs.iter().map(|l| l.as_str()).collect();
        println!("语言检测: {}", labels.join(", "));
    }

    // 2. 配置文件版本读取
    let versions = quanttide_devops::source::config_file::read_config_versions(&repo_path);
    if versions.is_empty() {
        println!("版本号:   (无配置文件)");
    } else {
        println!("版本号:");
        for (file, ver) in &versions {
            let label = ver.as_deref().unwrap_or("(未设置)");
            println!("   {:<20} {}", file, label);
        }
    }

    // 3. 契约自动检测（Contract::auto_detect 扫描目录结构）
    println!("\n契约自动检测:");
    let contract = quanttide_devops::contract::Contract::auto_detect(&repo_path);
    println!("  scopes: {} 个", contract.scopes.len());
    for scope in &contract.scopes {
        println!(
            "    {:<12} dir: {:<20} tool: {}",
            scope.name,
            scope.dir,
            scope.build_tool.as_str(),
        );
    }
}

/// 生成一个带 Cargo.toml 和 pyproject.toml 的临时目录以便演示。
fn demo_tempdir() -> std::path::PathBuf {
    let dir = tempfile::tempdir().expect("创建临时目录失败");
    let path = dir.path().to_path_buf();

    std::fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.2.0\"\n",
    )
    .expect("写入 Cargo.toml 失败");

    std::fs::write(
        path.join("pyproject.toml"),
        "[project]\nname = \"demo-py\"\nversion = \"0.1.0\"\n",
    )
    .expect("写入 pyproject.toml 失败");

    // 临时目录会在 dir 析构时删除，但 main 结束后才析构，所以没问题
    // 为防止编译器警告，泄漏 dir 的所有权
    std::mem::forget(dir);
    path
}
