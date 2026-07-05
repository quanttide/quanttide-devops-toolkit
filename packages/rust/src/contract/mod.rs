pub mod core;
/// 契约模型四维架构：Stages / Platforms / Sources / Scopes。
///
/// 参考：<https://github.com/quanttide/quanttide-essay-of-devops/blob/main/contract/index.md>
pub mod error;
pub mod platform;
pub mod scope;
pub mod source;
pub mod stage;
pub mod version;

pub use core::Contract;
pub use error::ContractError;
pub use platform::{Pipeline, Platform, Registry, SourceControl};
pub use scope::{BuildTool, Language, Scope};
pub use source::{Source, SourceType};
pub use stage::{Stage, StageBuild, StageRelease, StageTest};
pub use version::{
    VersionState, check_version_consistency, normalize_version, validate_version, verify_version,
};

use std::path::Path;

/// 从 `.quanttide/devops/contract.yaml` 加载契约。
pub fn load(repo_path: &Path) -> Result<Contract, ContractError> {
    let path = repo_path.join(".quanttide/devops/contract.yaml");
    let content = std::fs::read_to_string(&path)?;
    let mut contract: Contract = load_from_str(&content)?;
    // Auto 类型展开为具体类型
    if contract.sources.version.source_type == SourceType::Auto {
        contract.sources.version.source_type = SourceType::detect(repo_path);
    }
    Ok(contract)
}

/// 加载契约，不存在时自动推测。
pub fn load_or_default(repo_path: &Path) -> Contract {
    load(repo_path).unwrap_or_else(|_| Contract::auto_detect(repo_path))
}

/// 从 YAML 字符串解析契约。
pub fn load_from_str(s: &str) -> Result<Contract, ContractError> {
    serde_yaml::from_str::<Contract>(s).map_err(|e| ContractError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_str_valid() {
        let yaml = r#"
stages:
  test:
    threshold: 80

scopes:
  cli:
    dir: src/cli
"#;
        let c = load_from_str(yaml).unwrap();
        assert_eq!(c.scopes.len(), 1);
        assert_eq!(c.scopes[0].name, "cli");
        assert_eq!(c.scopes[0].dir, "src/cli");
        assert_eq!(c.stages.test.threshold, 80.0);
    }

    #[test]
    fn test_load_from_str_empty() {
        let c = load_from_str("").unwrap();
        assert!(c.scopes.is_empty());
    }

    #[test]
    fn test_load_from_str_invalid() {
        let err = load_from_str("invalid: [").unwrap_err();
        assert!(err.to_string().contains("解析失败"));
    }
}
