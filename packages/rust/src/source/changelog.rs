use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// CHANGELOG 操作错误。
#[derive(Debug)]
pub enum ChangelogError {
    /// 文件读取失败。
    Io(std::io::Error),
    /// 解析失败。
    Parse(String),
}

impl std::fmt::Display for ChangelogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "读取 CHANGELOG 失败: {}", e),
            Self::Parse(e) => write!(f, "解析 CHANGELOG 失败: {}", e),
        }
    }
}

impl std::error::Error for ChangelogError {}

impl From<std::io::Error> for ChangelogError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Changelog 结构
// ═══════════════════════════════════════════════════════════════════════

/// CHANGELOG 解析结果。封装 `parse-changelog` 提供便捷方法。
///
/// 内部持有原始文本 + 解析后的有序 Map（版本号 → Release）。
#[derive(Debug)]
pub struct Changelog {
    #[allow(dead_code)]
    /// 原始文本，保障解析结果的引用有效性。
    raw: String,
    /// 解析后的版本 → Release 有序 Map。
    ///
    /// # Safety
    ///
    /// `inner` 中的 `&str` 引用指向 `self.raw` 的堆内存。
    /// `raw` 和 `inner` 始终一起移动和释放，因此引用始终有效。
    inner: parse_changelog::Changelog<'static>,
}

impl Changelog {
    /// 从文件路径解析 CHANGELOG。
    pub fn from_path(path: &Path) -> Result<Self, ChangelogError> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_str(&raw)
    }

    /// 从字符串解析 CHANGELOG。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, ChangelogError> {
        let raw = s.to_string();
        // Safety: inner 的 &str 引用指向 raw。
        // raw 和 inner 始终一起移动和释放，引用在 Changelog 存活期间始终有效。
        let inner =
            parse_changelog::parse(&raw).map_err(|e| ChangelogError::Parse(e.to_string()))?;
        // SAFETY: inner 的 &str 引用指向 raw。raw 和 inner 始终一起移动和释放。
        let inner: parse_changelog::Changelog<'static> = unsafe { std::mem::transmute(inner) };
        Ok(Self { raw, inner })
    }

    /// 获取指定版本的 release notes（用于 GitHub Release body）。
    pub fn release_notes<'a>(&'a self, version: &str) -> Option<&'a str> {
        self.inner.get(version).map(|r| r.notes)
    }

    /// 检查指定版本是否存在于 CHANGELOG 中。
    pub fn contains_version(&self, version: &str) -> bool {
        self.inner.contains_key(version)
    }

    /// 获取最新发布的版本号（即文件中第一个版本）。
    pub fn latest_version(&self) -> Option<&str> {
        self.inner.keys().next().copied()
    }

    /// 获取所有版本号列表（保持文件中先后顺序）。
    pub fn versions(&self) -> Vec<&str> {
        self.inner.keys().copied().collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_changelog() -> &'static str {
        "\
# Changelog

## [0.1.2] - 2026-07-02

### Changed
- Refactored model into modules.

## [0.1.1] - 2026-07-02

### Added
- Version utility functions.

## [0.1.0] - 2026-07-02

### Added
- Initial release.
"
    }

    #[test]
    fn test_release_notes_existing() {
        let cl = Changelog::from_str(sample_changelog()).unwrap();
        let notes = cl.release_notes("0.1.1").unwrap();
        assert!(notes.contains("Version utility functions"));
    }

    #[test]
    fn test_release_notes_not_found() {
        let cl = Changelog::from_str(sample_changelog()).unwrap();
        assert!(cl.release_notes("9.9.9").is_none());
    }

    #[test]
    fn test_contains_version() {
        let cl = Changelog::from_str(sample_changelog()).unwrap();
        assert!(cl.contains_version("0.1.0"));
        assert!(!cl.contains_version("0.2.0"));
    }

    #[test]
    fn test_latest_version() {
        let cl = Changelog::from_str(sample_changelog()).unwrap();
        assert_eq!(cl.latest_version(), Some("0.1.2"));
    }

    #[test]
    fn test_versions() {
        let cl = Changelog::from_str(sample_changelog()).unwrap();
        assert_eq!(cl.versions(), vec!["0.1.2", "0.1.1", "0.1.0"]);
    }

    #[test]
    fn test_empty_changelog() {
        let cl = Changelog::from_str("");
        assert!(cl.is_err());
        assert!(cl.unwrap_err().to_string().contains("no release note"));
    }

    #[test]
    fn test_from_path() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("CHANGELOG.md");
        std::fs::write(&path, sample_changelog()).unwrap();
        let cl = Changelog::from_path(&path).unwrap();
        assert_eq!(cl.latest_version(), Some("0.1.2"));
    }

    #[test]
    fn test_from_path_not_found() {
        let cl = Changelog::from_path(Path::new("/nonexistent/CHANGELOG.md"));
        assert!(cl.is_err());
    }

    #[test]
    fn test_changelog_strips_v_prefix() {
        let s = "\
## v0.1.0 - 2026-01-01

### Added
- Something.
";
        // v 前缀被解析器剥离，版本 key 统一不带 v
        let cl = Changelog::from_str(s).unwrap();
        assert!(cl.contains_version("0.1.0"));
        assert!(!cl.contains_version("v0.1.0"));
    }

    #[test]
    fn test_scoped_pure_version() {
        // CHANGELOG 中写纯版本号，查询也传纯版本号
        let s = "\
## [0.1.0] - 2026-01-01

### Added
- CLI release.
";
        let cl = Changelog::from_str(s).unwrap();
        assert!(cl.contains_version("0.1.0"));
        // scope 前缀版本不应匹配 CHANGELOG 中的纯版本
        assert!(!cl.contains_version("cli/0.1.0"));
    }
}
