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
