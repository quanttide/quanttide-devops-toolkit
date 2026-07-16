//! 来源模块：从文件系统、Git 等来源读取项目元数据。
//!
//! 子模块覆盖 changelog、配置文件、Git 操作和 ROADMAP 解析。

pub mod changelog;
pub mod config_file;
pub mod git;
pub mod roadmap;
