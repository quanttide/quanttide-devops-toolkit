use std::fmt;
use std::io;

/// 契约操作错误。
#[derive(Debug)]
pub enum ContractError {
    /// 配置文件 I/O 错误。
    Io(io::Error),
    /// YAML 解析错误。
    Parse(String),
    /// 配置文件不存在。
    NotFound,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "读取契约文件失败: {}", e),
            Self::Parse(msg) => write!(f, "契约 YAML 解析失败: {}", msg),
            Self::NotFound => write!(f, "契约文件不存在"),
        }
    }
}

impl std::error::Error for ContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ContractError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
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
