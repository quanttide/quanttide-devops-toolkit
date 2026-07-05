//! 从文件系统检测编程语言，作为语言的事实源。
//!
//! # 事实源定位
//!
//! 本模块回答"目录下是什么语言的项目"，通过检测标志文件来判断。
//! 检测结果可能被 `contract::core::resolve_language` 使用（scope 未声明语言时兜底）。

use std::path::Path;

use crate::contract::Language;

/// 根据目录下的标志文件推测编程语言。
///
/// ```
/// use std::path::Path;
/// use quanttide_devops::source::language::detect;
///
/// let lang = detect(Path::new("/tmp/nonexistent"));
/// assert!(matches!(lang, quanttide_devops::contract::Language::Unknown(_)));
/// ```
pub fn detect(dir: &Path) -> Language {
    if dir.join("Cargo.toml").exists() {
        Language::Rust
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        Language::Python
    } else if dir.join("go.mod").exists() {
        Language::Go
    } else if dir.join("pubspec.yaml").exists() {
        Language::Dart
    } else if dir.join("package.json").exists() {
        Language::TypeScript
    } else {
        Language::Unknown("无法识别".into())
    }
}
