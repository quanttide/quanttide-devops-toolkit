/// 发布生命周期状态。
///
/// 每个发布版本依次经历 `Staged → Published → Retired`，或在任一步被 `Cancelled` 终止。
///
/// ```
/// use quanttide_devops::release::ReleaseStatus;
///
/// let s = ReleaseStatus::Staged;
/// assert_eq!(format!("{:?}", s), "Staged");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseStatus {
    /// 已标记预发布（rc 版本），待验证
    Staged,
    /// 已正式发布上线
    Published,
    /// 已取消（审计用途，不清理 tag）
    Cancelled,
    /// 已退役（Published 版本标记为不可用）
    Retired,
}

/// 发布版本记录。
///
/// 每次发布操作生成一条记录，包含版本号、状态和双时间戳。
///
/// ```
/// use quanttide_devops::release::{ReleaseRecord, ReleaseStatus};
///
/// let r = ReleaseRecord::new("v1.0.0", "uuid-123".into(), "1700000000".into());
/// assert_eq!(r.version, "v1.0.0");
/// assert_eq!(r.status, ReleaseStatus::Staged);
/// assert_eq!(r.created_at, r.updated_at);
/// ```
#[derive(Debug, Clone)]
pub struct ReleaseRecord {
    /// 记录 ID（由调用方生成，如 UUID）
    pub id: String,
    /// 版本号（如 `v1.0.0`、`cli/v0.3.0`）
    pub version: String,
    /// 当前状态
    pub status: ReleaseStatus,
    /// 创建时间（Unix 秒时间戳，由调用方传入以保持幂等）
    pub created_at: String,
    /// 更新时间（最近一次状态变更的时间戳）
    pub updated_at: String,
}

impl ReleaseRecord {
    /// 创建一个新的 Staged 状态记录。
    ///
    /// `new` 不依赖外部 crate（无 uuid、无系统时间调用），由调用方传入 ID 和时间戳。
    pub fn new(version: &str, id: String, now: String) -> Self {
        Self {
            id,
            version: version.to_string(),
            status: ReleaseStatus::Staged,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// 状态转换错误。
///
/// 表示版本不满足执行某操作的前置状态要求。
///
/// ```
/// use quanttide_devops::release::TransitionError;
///
/// let e = TransitionError::NotStaged("v1.0.0".into());
/// assert!(e.to_string().contains("Staged"));
/// ```
#[derive(Debug)]
pub enum TransitionError {
    /// 版本不处于 Staged 状态
    NotStaged(String),
    /// 版本不处于 Published 状态
    NotPublished(String),
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStaged(v) => write!(f, "版本 {} 不处于 Staged 状态", v),
            Self::NotPublished(v) => write!(f, "版本 {} 不处于 Published 状态", v),
        }
    }
}

impl std::error::Error for TransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_debug() {
        assert_eq!(format!("{:?}", ReleaseStatus::Staged), "Staged");
        assert_eq!(format!("{:?}", ReleaseStatus::Published), "Published");
        assert_eq!(format!("{:?}", ReleaseStatus::Cancelled), "Cancelled");
        assert_eq!(format!("{:?}", ReleaseStatus::Retired), "Retired");
    }

    #[test]
    fn test_status_clone_eq() {
        assert_eq!(ReleaseStatus::Staged, ReleaseStatus::Staged);
        assert_ne!(ReleaseStatus::Staged, ReleaseStatus::Published);
    }

    #[test]
    fn test_record_new_staged() {
        let r = ReleaseRecord::new("v1.0.0", "id-1".into(), "100".into());
        assert_eq!(r.version, "v1.0.0");
        assert_eq!(r.id, "id-1");
        assert_eq!(r.status, ReleaseStatus::Staged);
        assert_eq!(r.created_at, r.updated_at);
    }

    #[test]
    fn test_record_clone() {
        let a = ReleaseRecord::new("v2.0.0", "id-2".into(), "200".into());
        let b = a.clone();
        assert_eq!(a.id, b.id);
        assert_eq!(a.version, b.version);
    }

    #[test]
    fn test_transition_error_display() {
        let e = TransitionError::NotStaged("v1.0.0".into());
        assert!(e.to_string().contains("Staged"));

        let e = TransitionError::NotPublished("v1.0.0".into());
        assert!(e.to_string().contains("Published"));
    }

    #[test]
    fn test_transition_error_debug() {
        let e = TransitionError::NotStaged("v1.0.0".into());
        assert!(format!("{:?}", e).contains("NotStaged"));
    }
}
