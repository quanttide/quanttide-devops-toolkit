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
}
