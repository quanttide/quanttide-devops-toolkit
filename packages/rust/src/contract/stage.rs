use serde::{Deserialize, Serialize};

// ── Stages（时序维度）────────────────────────────────────────────────

/// 生命周期阶段配置。
///
/// 不规定"怎么做"，只规定"什么时候检查什么"。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stage {
    #[serde(default)]
    pub build: StageBuild,
    #[serde(default)]
    pub test: StageTest,
    #[serde(default)]
    pub release: StageRelease,
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            build: StageBuild { command: None },
            test: StageTest {
                command: None,
                threshold: 70.0,
            },
            release: StageRelease {
                changelog: "CHANGELOG.md".into(),
                pre_publish: Vec::new(),
            },
        }
    }
}

/// 构建阶段。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageBuild {
    #[serde(default)]
    pub command: Option<String>,
}

/// 测试阶段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageTest {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

impl Default for StageTest {
    fn default() -> Self {
        Self {
            command: None,
            threshold: 70.0,
        }
    }
}

const fn default_threshold() -> f64 {
    70.0
}

/// 发布阶段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StageRelease {
    #[serde(default = "default_changelog")]
    pub changelog: String,
    #[serde(default)]
    pub pre_publish: Vec<String>,
}

fn default_changelog() -> String {
    "CHANGELOG.md".into()
}

impl Default for StageRelease {
    fn default() -> Self {
        Self {
            changelog: "CHANGELOG.md".into(),
            pre_publish: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_build_default() {
        let s = StageBuild::default();
        assert_eq!(s.command, None);
    }

    #[test]
    fn test_stage_test_default() {
        let s = StageTest::default();
        assert_eq!(s.command, None);
        assert_eq!(s.threshold, 70.0);
    }

    #[test]
    fn test_default_threshold_fn() {
        assert_eq!(default_threshold(), 70.0);
    }

    #[test]
    fn test_default_changelog_fn() {
        assert_eq!(default_changelog(), "CHANGELOG.md");
    }
}
