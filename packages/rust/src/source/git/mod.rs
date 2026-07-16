//! Git 相关模块：仓库检测、tag 读取与版本管理。
//!
//! - [`repo`] — 仓库查询（`is_git_repo`）
//! - [`tag`] — tag 操作（`TagSource`、`GixTagSource`、过滤/排序/创建/推送）

pub mod repo;
pub mod submodule;
pub mod tag;

pub use repo::is_git_repo;
pub use submodule::{AggregateStatus, HealthIssue, RepoState, Submodule, SubmoduleStatus, describe_issue, fmt_oid};
pub use tag::{
    filter_latest_tag, filter_latest_version, filter_tags_by_scope, latest_tag,
    latest_tag_with, latest_version, latest_version_with, parse_semver_tag,
    tags_for_scope, tags_for_scope_with, GixTagSource, TagError, TagSource,
};
