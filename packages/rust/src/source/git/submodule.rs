//! 子模块数据模型。
//!
//! 提供子模块的枚举、结构体等纯数据模型。I/O 操作在 CLI 层。

/// 截断 OID 到 7 字符显示。
pub fn fmt_oid(id: &gix::ObjectId) -> String {
    gix::hash::Prefix::new(id, 7)
        .map(|p| p.to_string())
        .unwrap_or_else(|_| id.to_string())
}

/// 子模块同步状态。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum SubmoduleStatus {
    Clean,
    AheadOfParent,
    BehindRemote,
    Detached,
    Dirty,
    Orphaned,
    Uninitialized,
}

impl SubmoduleStatus {
    /// 优先级（数值越小越需要关注）。
    pub fn priority(&self) -> u8 {
        match self {
            Self::Dirty => 0,
            Self::Orphaned => 1,
            Self::Detached => 2,
            Self::Uninitialized => 3,
            Self::BehindRemote => 4,
            Self::AheadOfParent => 5,
            Self::Clean => 6,
        }
    }
}

/// 子模块信息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct Submodule {
    pub name: String,
    pub path: std::path::PathBuf,
    pub url: String,
    pub tracked_branch: String,
    pub parent_pointer: gix::ObjectId,
    pub local_head: gix::ObjectId,
    pub remote_head: gix::ObjectId,
    pub status: SubmoduleStatus,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub remote_unreachable: bool,
}

/// 仓库子模块快照。
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoState {
    pub root_path: std::path::PathBuf,
    pub submodules: Vec<Submodule>,
    pub total: usize,
    pub clean_count: usize,
    pub needs_attention: Vec<String>,
}

/// 子模块状态聚合统计。
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AggregateStatus {
    pub total: usize,
    pub clean: usize,
    pub ahead_of_parent: usize,
    pub behind_remote: usize,
    pub detached: usize,
    pub dirty: usize,
    pub orphaned: usize,
    pub uninitialized: usize,
}

impl AggregateStatus {
    /// 从子模块列表聚合统计。
    pub fn from_submodules(submodules: &[Submodule]) -> Self {
        let mut clean = 0;
        let mut ahead = 0;
        let mut behind = 0;
        let mut detached = 0;
        let mut dirty = 0;
        let mut orphaned = 0;
        let mut uninit = 0;
        for sm in submodules {
            match sm.status {
                SubmoduleStatus::Clean => clean += 1,
                SubmoduleStatus::AheadOfParent => ahead += 1,
                SubmoduleStatus::BehindRemote => behind += 1,
                SubmoduleStatus::Detached => detached += 1,
                SubmoduleStatus::Dirty => dirty += 1,
                SubmoduleStatus::Orphaned => orphaned += 1,
                SubmoduleStatus::Uninitialized => uninit += 1,
            }
        }
        AggregateStatus {
            total: submodules.len(),
            clean,
            ahead_of_parent: ahead,
            behind_remote: behind,
            detached,
            dirty,
            orphaned,
            uninitialized: uninit,
        }
    }
}

/// 子模块健康问题描述。
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub submodule_name: String,
    pub status: String,
    pub description: String,
    pub suggested_action: String,
}

/// 根据子模块状态返回（问题描述，建议操作）。
pub fn describe_issue(status: &SubmoduleStatus) -> (String, String) {
    match status {
        SubmoduleStatus::AheadOfParent => (
            "本地领先于父仓库记录".into(),
            "运行 sync_to_parent 更新父仓库指针".into(),
        ),
        SubmoduleStatus::BehindRemote => (
            "远程有更新，本地落后".into(),
            "运行 code sync 获取最新代码".into(),
        ),
        SubmoduleStatus::Detached => (
            "处于游离 HEAD 状态".into(),
            "运行 checkout_branch 切换到跟踪分支".into(),
        ),
        SubmoduleStatus::Dirty => ("有未提交的修改".into(), "提交或 stash 当前修改".into()),
        SubmoduleStatus::Orphaned => (
            "父仓库记录的 commit 在远程已不存在".into(),
            "需手动干预".into(),
        ),
        SubmoduleStatus::Uninitialized => ("尚未初始化".into(), "运行 init 初始化子模块".into()),
        SubmoduleStatus::Clean => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_oid_null() {
        let null = gix::ObjectId::null(gix::hash::Kind::Sha1);
        assert_eq!(fmt_oid(&null).len(), 7);
    }

    #[test]
    fn test_submodule_status_priority() {
        assert!(SubmoduleStatus::Dirty.priority() < SubmoduleStatus::Clean.priority());
    }

    #[test]
    fn test_aggregate_from_submodules() {
        let sm = |s: SubmoduleStatus| Submodule {
            name: "test".into(),
            path: "test".into(),
            url: "".into(),
            tracked_branch: "main".into(),
            parent_pointer: gix::ObjectId::null(gix::hash::Kind::Sha1),
            local_head: gix::ObjectId::null(gix::hash::Kind::Sha1),
            remote_head: gix::ObjectId::null(gix::hash::Kind::Sha1),
            status: s,
            ahead_count: 0,
            behind_count: 0,
            remote_unreachable: false,
        };
        let subs = vec![sm(SubmoduleStatus::Clean), sm(SubmoduleStatus::Dirty)];
        let agg = AggregateStatus::from_submodules(&subs);
        assert_eq!(agg.total, 2);
        assert_eq!(agg.clean, 1);
        assert_eq!(agg.dirty, 1);
    }

    #[test]
    fn test_describe_issue_all_variants() {
        for variant in &[
            SubmoduleStatus::AheadOfParent,
            SubmoduleStatus::BehindRemote, SubmoduleStatus::Detached,
            SubmoduleStatus::Dirty, SubmoduleStatus::Orphaned,
            SubmoduleStatus::Uninitialized,
        ] {
            let (desc, action) = describe_issue(variant);
            assert!(!desc.is_empty());
            assert!(!action.is_empty());
        }
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn test_describe_issue_clean_panics() {
        describe_issue(&SubmoduleStatus::Clean);
    }

    #[test]
    fn test_all_priorities_are_unique() {
        let mut prios: Vec<u8> = vec![
            SubmoduleStatus::Clean.priority(),
            SubmoduleStatus::AheadOfParent.priority(),
            SubmoduleStatus::BehindRemote.priority(),
            SubmoduleStatus::Detached.priority(),
            SubmoduleStatus::Dirty.priority(),
            SubmoduleStatus::Orphaned.priority(),
            SubmoduleStatus::Uninitialized.priority(),
        ];
        prios.sort();
        prios.dedup();
        assert_eq!(prios.len(), 7);
    }

    #[test]
    fn test_clean_is_lowest_priority() {
        let prios = [
            SubmoduleStatus::Clean, SubmoduleStatus::AheadOfParent,
            SubmoduleStatus::BehindRemote, SubmoduleStatus::Detached,
            SubmoduleStatus::Dirty, SubmoduleStatus::Orphaned,
            SubmoduleStatus::Uninitialized,
        ];
        let max = prios.iter().map(|s| s.priority()).max().unwrap();
        assert_eq!(max, SubmoduleStatus::Clean.priority());
    }
}
