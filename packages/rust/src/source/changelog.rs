use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// CHANGELOG 操作错误。
#[derive(Debug)]
pub enum ChangelogError {
    /// 文件读取失败。
    Io(std::io::Error),
    /// 文件写入失败。
    File(String),
    /// git 命令失败。
    Git(String),
    /// 解析失败。
    Parse(String),
}

impl std::fmt::Display for ChangelogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "读取 CHANGELOG 失败: {}", e),
            Self::File(e) => write!(f, "文件写入失败: {}", e),
            Self::Git(e) => write!(f, "git 命令失败: {}", e),
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
// Git 日志收集
// ═══════════════════════════════════════════════════════════════════════

/// 收集 git 提交记录。
///
/// `from_tag = Some(tag)` 时范围是 `tag..HEAD`，`None` 时返回全部提交。
pub fn collect_git_log(repo_path: &Path, from_tag: Option<&str>) -> Result<String, ChangelogError> {
    let range = match from_tag {
        Some(tag) => format!("{}..HEAD", tag),
        None => "HEAD".to_string(),
    };
    let args = ["log", "--oneline", &range];

    let output = std::process::Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| ChangelogError::Git(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ChangelogError::Git(stderr));
    }

    let log = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if log.is_empty() {
        return Err(ChangelogError::Git("没有新的提交记录".into()));
    }

    Ok(log)
}

// ═══════════════════════════════════════════════════════════════════════
// LLM Prompt 构建
// ═══════════════════════════════════════════════════════════════════════

/// 根据 git 提交记录构建 CHANGELOG 生成 prompt。
///
/// 输出固定的分类规则（Added / Changed / Fixed / Removed），要求 LLM 用中文生成。
pub fn build_changelog_prompt(git_log: &str, version: &str) -> String {
    format!(
        "根据以下 git 提交记录，为版本 {} 生成 CHANGELOG 条目。\n\
         \n\
         要求：\n\
         1. 按 Added / Changed / Fixed / Removed 分类\n\
         2. 同类提交合并为概括性条目，不要逐条罗列\n\
         3. 用中文描述\n\
         4. 每类不超过 5 条\n\
         5. 仅输出内容，不要版本头部和日期\n\
         \n\
         提交记录：\n{}",
        version, git_log
    )
}

// ═══════════════════════════════════════════════════════════════════════
// CHANGELOG 条目追加
// ═══════════════════════════════════════════════════════════════════════

/// 标准化版本号：去掉 `v` 前缀、scope 前缀。
///
/// 例如 `"v0.1.0"` → `"0.1.0"`，`"cli/0.1.0"` → `"0.1.0"`。
fn normalize_version(version: &str) -> &str {
    let v = version.strip_prefix('v').unwrap_or(version);
    v.split('/').last().unwrap_or(v)
}

/// 向 CHANGELOG 文件追加新版本条目。
///
/// - 文件不存在时创建并写入头部 `# CHANGELOG\n`
/// - 版本已存在时跳过（返回 `Ok(false)`）
/// - 新条目插入到已有条目的最前面，放在已有第一个版本之前
pub fn append_entry(path: &Path, version: &str, content: &str) -> Result<bool, ChangelogError> {
    let bare = normalize_version(version);

    let today = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let entry = format!("\n## [{}] - {}\n\n{}\n", bare, today, content);

    if !path.exists() {
        // 文件不存在，创建并写入头部和条目
        let mut body = "# CHANGELOG\n".to_string();
        body.push_str(&entry);
        std::fs::write(path, &body).map_err(|e| ChangelogError::File(e.to_string()))?;
        return Ok(true);
    }

    let raw = std::fs::read_to_string(path)?;
    let changelog = Changelog::from_str(&raw).map_err(|_| {
        ChangelogError::File("已存在文件不是有效的 CHANGELOG 格式".into())
    })?;

    if changelog.contains_version(bare) {
        return Ok(false);
    }

    // 找到第一个版本条目的位置，在它前面插入
    let insert_at = if let Some(first) = changelog.latest_version() {
        // 查找第一个版本标题的位置
        let search = format!("## [{}]", first);
        let pos = raw.find(&search).unwrap_or(raw.len());
        pos
    } else {
        // 没有版本条目，追加到头部行之后
        let after_header = raw.find('\n').map(|i| i + 1).unwrap_or(raw.len());
        after_header
    };

    let mut body = String::with_capacity(raw.len() + entry.len());
    body.push_str(&raw[..insert_at]);
    body.push_str(&entry);
    body.push_str(&raw[insert_at..]);

    std::fs::write(path, &body).map_err(|e| ChangelogError::File(e.to_string()))?;
    Ok(true)
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
    fn test_changelog_error_display_git() {
        let err = ChangelogError::Git("rev-parse failed".into());
        assert!(err.to_string().contains("git 命令失败"));
    }

    #[test]
    fn test_changelog_error_display_file() {
        let err = ChangelogError::File("permission denied".into());
        assert!(err.to_string().contains("文件写入失败"));
    }

    #[test]
    fn test_build_prompt_contains_version() {
        let prompt = build_changelog_prompt("fix bug", "0.2.0");
        assert!(prompt.contains("0.2.0"));
    }

    #[test]
    fn test_build_prompt_contains_git_log() {
        let prompt = build_changelog_prompt("feat: add login\nfix: crash", "0.2.0");
        assert!(prompt.contains("feat: add login"));
        assert!(prompt.contains("fix: crash"));
    }

    #[test]
    fn test_normalize_version_strips_v() {
        assert_eq!(normalize_version("v0.1.0"), "0.1.0");
    }

    #[test]
    fn test_normalize_version_strips_scope() {
        assert_eq!(normalize_version("cli/0.1.0"), "0.1.0");
    }

    #[test]
    fn test_normalize_version_already_clean() {
        assert_eq!(normalize_version("0.1.0"), "0.1.0");
    }

    #[test]
    fn test_append_entry_creates_new_file() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("CHANGELOG.md");
        let ok = append_entry(&path, "0.1.0", "### Added\n- Initial release.").unwrap();
        assert!(ok);
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.starts_with("# CHANGELOG\n"));
        assert!(raw.contains("## [0.1.0]"));
        assert!(raw.contains("Initial release"));
    }

    #[test]
    fn test_append_entry_existing_version_skips() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("CHANGELOG.md");
        append_entry(&path, "0.1.0", "first").unwrap();
        let ok = append_entry(&path, "0.1.0", "second").unwrap();
        assert!(!ok);
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("first"));
        assert!(!raw.contains("second"));
    }

    #[test]
    fn test_append_entry_prepends() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("CHANGELOG.md");
        append_entry(&path, "0.1.0", "first release").unwrap();
        append_entry(&path, "0.2.0", "second release").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        // 0.2.0 应该出现在 0.1.0 前面
        let pos1 = raw.find("0.2.0").unwrap();
        let pos2 = raw.find("0.1.0").unwrap();
        assert!(pos1 < pos2, "新版本应该插入到已有版本之前");
    }

    #[test]
    fn test_append_entry_strips_scope_version() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("CHANGELOG.md");
        append_entry(&path, "cli/0.1.0", "CLI release").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("cli/0.1.0"));
        assert!(raw.contains("## [0.1.0]"));
    }

    #[test]
    fn test_build_prompt_no_undefined_categories() {
        let prompt = build_changelog_prompt("something", "0.1.0");
        // 只应包含 Added / Changed / Fixed / Removed
        assert!(prompt.contains("Added"));
        assert!(prompt.contains("Changed"));
        assert!(prompt.contains("Fixed"));
        assert!(prompt.contains("Removed"));
        // 不包含未定义的关键词
        assert!(!prompt.contains("Deleted"));
        assert!(!prompt.contains("Deprecated"));
        assert!(!prompt.contains("Security"));
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
