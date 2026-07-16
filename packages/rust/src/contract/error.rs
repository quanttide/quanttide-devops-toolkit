//! 契约错误类型。
//!
//! 定义 `ContractError` 枚举，覆盖 I/O、解析、文件不存在等错误场景。

use thiserror::Error;
use std::io;

/// 契约操作错误。
#[derive(Error, Debug)]
pub enum ContractError {
    /// 配置文件 I/O 错误。
    #[error("读取契约文件失败: {0}")]
    Io(#[from] io::Error),
    /// YAML 解析错误。
    #[error("契约 YAML 解析失败: {0}")]
    Parse(String),
    /// 配置文件不存在。
    #[error("契约文件不存在")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::*;

    #[test]
    fn test_display_io() {
        let err = ContractError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        let msg = err.to_string();
        assert!(msg.contains("读取契约文件失败"));
    }

    #[test]
    fn test_display_parse() {
        let err = ContractError::Parse("syntax error".into());
        assert!(err.to_string().contains("YAML 解析失败"));
    }

    #[test]
    fn test_display_not_found() {
        let err = ContractError::NotFound;
        assert!(err.to_string().contains("契约文件不存在"));
    }

    #[test]
    fn test_source_io() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = ContractError::Io(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_source_other() {
        assert!(ContractError::NotFound.source().is_none());
        assert!(ContractError::Parse("x".into()).source().is_none());
    }

    #[test]
    fn test_from_io() {
        let inner = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: ContractError = inner.into();
        assert!(matches!(err, ContractError::Io(_)));
    }
}
