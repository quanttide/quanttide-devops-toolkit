pub(crate) mod parse;
pub(crate) mod validate;

// ═══════════════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════════════

/// ROADMAP 操作错误。
#[derive(Debug)]
pub enum RoadmapError {
    /// 文件读取失败。
    Io(std::io::Error),
    /// 解析失败（格式不符合预期）。
    Parse(String),
}

impl std::fmt::Display for RoadmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "读取 ROADMAP 失败: {}", e),
            Self::Parse(e) => write!(f, "解析 ROADMAP 失败: {}", e),
        }
    }
}

impl std::error::Error for RoadmapError {}

impl From<std::io::Error> for RoadmapError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 公共类型
// ═══════════════════════════════════════════════════════════════════════

/// 单个 checklist 条目。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapChecklistItem {
    /// 描述文本。
    pub description: String,
    /// 是否已勾选（`[x]`）。
    pub completed: bool,
}

/// 进度统计。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapProgress {
    /// 总条目数。
    pub total: usize,
    /// 已完成条目数。
    pub completed: usize,
}

impl RoadmapProgress {
    /// 完成百分比（0.0 ~ 100.0）。无条目时返回 100.0。
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }
}

/// 单版本的规划进度。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapVersion {
    /// 版本号（如 `"0.1.5"`）。
    pub version: String,
    /// 状态标签（如 `"待实施"`、`"已发布"`）。
    pub status: String,
    /// 已完成条目数。
    pub done: usize,
    /// 总条目数。
    pub total: usize,
    /// 分类分组：`(分类名, 条目列表)`。
    pub categories: Vec<(String, Vec<RoadmapChecklistItem>)>,
}

impl RoadmapVersion {
    /// 完成百分比（0.0 ~ 100.0）。无条目时返回 100.0。
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.done as f64 / self.total as f64) * 100.0
    }
}

/// 格式验证发现的单个问题。
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapIssue {
    /// 问题所在行号（1-based）。
    pub line: usize,
    /// 问题所属 scope。
    pub scope: String,
    /// 问题描述。
    pub message: String,
}

/// 解析后的 ROADMAP.md 文档。
#[derive(Debug, Clone, PartialEq)]
pub struct Roadmap {
    /// 原始文本。
    #[allow(dead_code)]
    pub(crate) raw: String,
    /// 所有版本区块（自上而下 = 最新优先）。
    pub(crate) versions: Vec<RoadmapVersion>,
}

// ═══════════════════════════════════════════════════════════════════════
// 公共访问器
// ═══════════════════════════════════════════════════════════════════════

impl Roadmap {
    /// 获取所有版本的规划进度。
    pub fn versions(&self) -> &[RoadmapVersion] {
        &self.versions
    }

    /// 总已完成条目数。
    pub fn total_done(&self) -> usize {
        self.versions.iter().map(|v| v.done).sum()
    }

    /// 总条目数。
    pub fn total_all(&self) -> usize {
        self.versions.iter().map(|v| v.total).sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 内部构建器
// ═══════════════════════════════════════════════════════════════════════

pub(crate) struct RoadmapVersionBuilder {
    pub(crate) version: String,
    pub(crate) status: String,
    pub(crate) categories: Vec<(String, Vec<RoadmapChecklistItem>)>,
}

impl RoadmapVersionBuilder {
    pub(crate) fn new(version: String, status: String) -> Self {
        Self {
            version,
            status,
            categories: Vec::new(),
        }
    }

    pub(crate) fn add_category(&mut self, name: String) {
        self.categories.push((name, Vec::new()));
    }

    pub(crate) fn add_issue(&mut self, completed: bool, description: String) {
        if let Some(last) = self.categories.last_mut() {
            last.1.push(RoadmapChecklistItem {
                description,
                completed,
            });
        }
    }

    pub(crate) fn build(self) -> RoadmapVersion {
        let total: usize = self.categories.iter().map(|(_, items)| items.len()).sum();
        let done: usize = self
            .categories
            .iter()
            .flat_map(|(_, items)| items)
            .filter(|i| i.completed)
            .count();

        RoadmapVersion {
            version: self.version,
            status: self.status,
            done,
            total,
            categories: self.categories,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Error display ────────────────────────────────────────────

    #[test]
    fn test_roadmap_error_display() {
        let err = RoadmapError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert!(err.to_string().contains("读取 ROADMAP 失败"));

        let err = RoadmapError::Parse("bad format".into());
        assert!(err.to_string().contains("解析 ROADMAP 失败"));
    }

    // ── RoadmapVersion percent ───────────────────────────────────

    #[test]
    fn test_version_percent() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 2,
            total: 4,
            categories: Vec::new(),
        };
        assert!((v.percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_empty() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 0,
            total: 0,
            categories: Vec::new(),
        };
        assert!((v.percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_all_done() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 3,
            total: 3,
            categories: Vec::new(),
        };
        assert!((v.percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_version_percent_none_done() {
        let v = RoadmapVersion {
            version: "0.1.0".into(),
            status: "test".into(),
            done: 0,
            total: 5,
            categories: Vec::new(),
        };
        assert!((v.percent() - 0.0).abs() < f64::EPSILON);
    }

    // ── RoadmapProgress percent ──────────────────────────────────

    #[test]
    fn test_progress_percent() {
        let p = RoadmapProgress {
            total: 4,
            completed: 2,
        };
        assert!((p.percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_empty() {
        let p = RoadmapProgress {
            total: 0,
            completed: 0,
        };
        assert!((p.percent() - 100.0).abs() < f64::EPSILON);
    }
}
