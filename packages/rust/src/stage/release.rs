/// 发布阶段的状态枚举。
///
/// 描述一个 scope 当前所处的发布生命周期阶段。
#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseStatus {
    /// 从未发布过（无匹配的 git tag）。
    Unreleased,
    /// 已发布且为最新状态，无新的未发布变更。
    Latest,
    /// 有自上次发布以来的未发布提交。
    Pending,
    /// tag 与配置文件版本不一致。
    Inconsistent,
    /// 无法确定状态（如 git 命令失败）。
    Unknown,
}

/// 返回状态的中文标签，用于命令行输出。
///
/// | 变体 | 标签 |
/// |---|---|
/// | `Unreleased` | 未发布 |
/// | `Latest` | 已是最新 |
/// | `Pending` | 待发布 |
/// | `Inconsistent` | 版本冲突 |
/// | `Unknown` | 状态未知 |
impl std::fmt::Display for ReleaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreleased => write!(f, "未发布"),
            Self::Latest => write!(f, "已是最新"),
            Self::Pending => write!(f, "待发布"),
            Self::Inconsistent => write!(f, "版本冲突"),
            Self::Unknown => write!(f, "状态未知"),
        }
    }
}

/// 发布阶段状态快照。
///
/// 记录一个 scope 在某个时刻的发布状态快照。
/// 与 [`VersionState`] 的关系：`VersionState` 聚焦版本号一致性，
/// `ReleaseState` 聚焦发布生命周期阶段。
#[derive(Debug)]
pub struct ReleaseState {
    /// 发布生命周期状态。
    pub status: ReleaseStatus,
    /// scope 名称。
    pub scope: String,
    /// scope 相对路径。
    pub scope_path: String,
    /// 当前最新 tag 版本号（若有）。
    pub current_version: Option<String>,
    /// 自最新 tag 以来的未发布提交数。
    pub pending_commits: usize,
    /// 变更日志路径。
    pub changelog: String,
    /// 版本一致性检查结果（空表示未检查或不适用）。
    pub version_consistent: Option<bool>,
}

impl ReleaseState {
    /// 创建一个 `ReleaseStateBuilder`，用链式调用替代 6 参数构造。
    ///
    /// ```
    /// use quanttide_devops::stage::release::{ReleaseState, ReleaseStatus};
    ///
    /// let s = ReleaseState::builder()
    ///     .status(ReleaseStatus::Pending)
    ///     .scope("cli")
    ///     .scope_path("src/cli")
    ///     .current_version("v1.2.3")
    ///     .pending_commits(3)
    ///     .version_consistent(true)
    ///     .build();
    /// assert_eq!(s.scope, "cli");
    /// assert_eq!(s.changelog, "CHANGELOG.md");
    /// ```
    pub fn builder() -> ReleaseStateBuilder {
        ReleaseStateBuilder::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Builder
// ═══════════════════════════════════════════════════════════════════════

/// [`ReleaseState`] 的构建器，替代多参数构造。
#[derive(Debug)]
pub struct ReleaseStateBuilder {
    status: Option<ReleaseStatus>,
    scope: Option<String>,
    scope_path: Option<String>,
    current_version: Option<String>,
    pending_commits: usize,
    version_consistent: Option<bool>,
    changelog: String,
}

impl Default for ReleaseStateBuilder {
    fn default() -> Self {
        Self {
            status: None,
            scope: None,
            scope_path: None,
            current_version: None,
            pending_commits: 0,
            version_consistent: None,
            changelog: "CHANGELOG.md".into(),
        }
    }
}

impl ReleaseStateBuilder {
    /// **必需**：发布生命周期状态。
    pub fn status(mut self, value: ReleaseStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// **必需**：scope 名称。
    pub fn scope(mut self, value: impl Into<String>) -> Self {
        self.scope = Some(value.into());
        self
    }

    /// **必需**：scope 相对路径。
    pub fn scope_path(mut self, value: impl Into<String>) -> Self {
        self.scope_path = Some(value.into());
        self
    }

    /// 当前最新 tag 版本号。
    pub fn current_version(mut self, value: impl Into<String>) -> Self {
        self.current_version = Some(value.into());
        self
    }

    /// 自最新 tag 以来的未发布提交数（默认 0）。
    pub fn pending_commits(mut self, value: usize) -> Self {
        self.pending_commits = value;
        self
    }

    /// 版本一致性检查结果。
    pub fn version_consistent(mut self, value: bool) -> Self {
        self.version_consistent = Some(value);
        self
    }

    /// 变更日志路径（默认 `"CHANGELOG.md"`）。
    pub fn changelog(mut self, value: impl Into<String>) -> Self {
        self.changelog = value.into();
        self
    }

    /// 构建 [`ReleaseState`]。
    ///
    /// # Panics
    ///
    /// 当 `status`、`scope` 或 `scope_path` 未设置时 panic。
    pub fn build(self) -> ReleaseState {
        ReleaseState {
            status: self.status.expect("ReleaseStateBuilder: status 是必需的"),
            scope: self.scope.expect("ReleaseStateBuilder: scope 是必需的"),
            scope_path: self
                .scope_path
                .expect("ReleaseStateBuilder: scope_path 是必需的"),
            current_version: self.current_version,
            pending_commits: self.pending_commits,
            changelog: self.changelog,
            version_consistent: self.version_consistent,
        }
    }
}

/// 多行报告格式，与平台 `status_to` 输出一致：
///
/// ```text
///   [scope]
///     状态:         已是最新
///     路径:         src/cli
///     最新版本:     v1.2.3
///     未发布提交:   0
///     变更日志:     CHANGELOG.md
///     版本一致:     是
/// ```
impl std::fmt::Display for ReleaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  [{}]", self.scope)?;
        writeln!(f, "    状态:         {}", self.status)?;
        writeln!(f, "    路径:         {}", self.scope_path)?;
        match &self.current_version {
            Some(v) => writeln!(f, "    最新版本:     {}", v)?,
            None => writeln!(f, "    最新版本:     (无)")?,
        }
        writeln!(f, "    未发布提交:   {}", self.pending_commits)?;
        writeln!(f, "    变更日志:     {}", self.changelog)?;
        match self.version_consistent {
            Some(true) => writeln!(f, "    版本一致:     是"),
            Some(false) => writeln!(f, "    版本一致:     否"),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ReleaseStatus ──────────────────────────────────────────────

    #[test]
    fn test_release_status_variants() {
        assert_ne!(ReleaseStatus::Unreleased, ReleaseStatus::Latest);
        assert_ne!(ReleaseStatus::Latest, ReleaseStatus::Pending);
        assert_ne!(ReleaseStatus::Pending, ReleaseStatus::Inconsistent);
        assert_ne!(ReleaseStatus::Inconsistent, ReleaseStatus::Unknown);
    }

    #[test]
    fn test_release_status_debug() {
        let s = format!("{:?}", ReleaseStatus::Unreleased);
        assert_eq!(s, "Unreleased");
    }

    #[test]
    fn test_release_status_display() {
        assert_eq!(format!("{}", ReleaseStatus::Unreleased), "未发布");
        assert_eq!(format!("{}", ReleaseStatus::Latest), "已是最新");
        assert_eq!(format!("{}", ReleaseStatus::Pending), "待发布");
        assert_eq!(format!("{}", ReleaseStatus::Inconsistent), "版本冲突");
        assert_eq!(format!("{}", ReleaseStatus::Unknown), "状态未知");
    }

    // ── ReleaseState ───────────────────────────────────────────────

    #[test]
    fn test_release_state_unreleased() {
        let state = ReleaseState {
            status: ReleaseStatus::Unreleased,
            scope: "cli".into(),
            scope_path: "src/cli".into(),
            current_version: None,
            pending_commits: 0,
            changelog: "CHANGELOG.md".into(),
            version_consistent: None,
        };
        assert_eq!(state.status, ReleaseStatus::Unreleased);
        assert!(state.current_version.is_none());
        assert_eq!(state.pending_commits, 0);
    }

    #[test]
    fn test_release_state_pending() {
        let state = ReleaseState {
            status: ReleaseStatus::Pending,
            scope: "core".into(),
            scope_path: "packages/core".into(),
            current_version: Some("v1.2.3".into()),
            pending_commits: 5,
            changelog: "CHANGELOG.md".into(),
            version_consistent: Some(true),
        };
        assert_eq!(state.status, ReleaseStatus::Pending);
        assert_eq!(state.current_version.as_deref(), Some("v1.2.3"));
        assert_eq!(state.pending_commits, 5);
        assert_eq!(state.version_consistent, Some(true));
    }

    #[test]
    fn test_release_state_latest() {
        let state = ReleaseState {
            status: ReleaseStatus::Latest,
            scope: "(root)".into(),
            scope_path: ".".into(),
            current_version: Some("v5.0.0".into()),
            pending_commits: 0,
            changelog: "CHANGELOG.md".into(),
            version_consistent: Some(true),
        };
        assert_eq!(state.status, ReleaseStatus::Latest);
        assert_eq!(state.pending_commits, 0);
    }

    #[test]
    fn test_release_state_inconsistent() {
        let state = ReleaseState {
            status: ReleaseStatus::Inconsistent,
            scope: "web".into(),
            scope_path: "src/web".into(),
            current_version: Some("v2.0.0".into()),
            pending_commits: 3,
            changelog: "CHANGELOG.md".into(),
            version_consistent: Some(false),
        };
        assert_eq!(state.status, ReleaseStatus::Inconsistent);
        assert_eq!(state.version_consistent, Some(false));
    }

    #[test]
    fn test_release_state_unknown() {
        let state = ReleaseState {
            status: ReleaseStatus::Unknown,
            scope: "service".into(),
            scope_path: "apps/service".into(),
            current_version: None,
            pending_commits: 0,
            changelog: "CHANGELOG.md".into(),
            version_consistent: None,
        };
        assert_eq!(state.status, ReleaseStatus::Unknown);
    }

    #[test]
    fn test_release_state_builder_minimal() {
        let s = ReleaseState::builder()
            .status(ReleaseStatus::Unreleased)
            .scope("cli")
            .scope_path("src/cli")
            .build();
        assert_eq!(s.scope, "cli");
        assert_eq!(s.scope_path, "src/cli");
        assert_eq!(s.status, ReleaseStatus::Unreleased);
        assert!(s.current_version.is_none());
        assert_eq!(s.pending_commits, 0);
        assert_eq!(s.changelog, "CHANGELOG.md");
        assert!(s.version_consistent.is_none());
    }

    #[test]
    fn test_release_state_builder_full() {
        let s = ReleaseState::builder()
            .status(ReleaseStatus::Pending)
            .scope("(root)")
            .scope_path(".")
            .current_version("v2.0.0")
            .pending_commits(5)
            .version_consistent(false)
            .build();
        assert_eq!(s.scope, "(root)");
        assert_eq!(s.current_version.as_deref(), Some("v2.0.0"));
        assert_eq!(s.pending_commits, 5);
        assert_eq!(s.version_consistent, Some(false));
    }

    #[test]
    fn test_release_state_display() {
        let state = ReleaseState {
            status: ReleaseStatus::Latest,
            scope: "qtcloud-core".into(),
            scope_path: "apps/qtcloud-core".into(),
            current_version: Some("v2.1.0".into()),
            pending_commits: 0,
            changelog: "CHANGELOG.md".into(),
            version_consistent: Some(true),
        };
        assert_eq!(
            format!("{}", state),
            "  [qtcloud-core]\n    状态:         已是最新\n    路径:         apps/qtcloud-core\n    最新版本:     v2.1.0\n    未发布提交:   0\n    变更日志:     CHANGELOG.md\n    版本一致:     是\n"
        );

        let state = ReleaseState {
            status: ReleaseStatus::Unreleased,
            scope: "(root)".into(),
            scope_path: ".".into(),
            current_version: None,
            pending_commits: 0,
            changelog: "CHANGELOG.md".into(),
            version_consistent: None,
        };
        assert_eq!(
            format!("{}", state),
            "  [(root)]\n    状态:         未发布\n    路径:         .\n    最新版本:     (无)\n    未发布提交:   0\n    变更日志:     CHANGELOG.md\n"
        );

        let state = ReleaseState {
            status: ReleaseStatus::Inconsistent,
            scope: "web".into(),
            scope_path: "src/web".into(),
            current_version: Some("v2.0.0".into()),
            pending_commits: 3,
            changelog: "CHANGELOG.md".into(),
            version_consistent: Some(false),
        };
        assert_eq!(
            format!("{}", state),
            "  [web]\n    状态:         版本冲突\n    路径:         src/web\n    最新版本:     v2.0.0\n    未发布提交:   3\n    变更日志:     CHANGELOG.md\n    版本一致:     否\n"
        );
    }
}
