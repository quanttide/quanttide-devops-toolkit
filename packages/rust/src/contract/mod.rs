/// 契约模型四维架构：Stages / Platforms / Sources / Scopes。
///
/// 参考：<https://github.com/quanttide/quanttide-essay-of-devops/blob/main/contract/index.md>
pub mod error;
pub mod model;

pub use error::ContractError;
pub use model::{
    BuildTool, Contract, Language, Pipeline, Platform, Registry, Scope, Source, SourceControl,
    SourceType, Stage, StageBuild, StageRelease, StageTest, VersionSource,
    detect_language_by_files,
};

use std::path::Path;

/// 从 `.quanttide/devops/contract.yaml` 加载契约。
pub fn load(repo_path: &Path) -> Result<Contract, ContractError> {
    let path = repo_path.join(".quanttide/devops/contract.yaml");
    let content = std::fs::read_to_string(&path)?;
    load_from_str(&content)
}

/// 从 YAML 字符串解析契约。
pub fn load_from_str(s: &str) -> Result<Contract, ContractError> {
    serde_yaml::from_str::<Contract>(s).map_err(|e| ContractError::Parse(e.to_string()))
}
