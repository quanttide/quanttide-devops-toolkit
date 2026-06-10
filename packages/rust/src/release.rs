#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseStatus {
    Staged,
    Published,
    Cancelled,
    Retired,
}

#[derive(Debug, Clone)]
pub struct ReleaseRecord {
    pub id: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl ReleaseRecord {
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

#[derive(Debug)]
pub enum TransitionError {
    NotStaged(String),
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
