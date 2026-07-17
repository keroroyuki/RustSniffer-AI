//! 统一错误类型
//!
//! 使用 thiserror 定义 SnifferError 枚举，覆盖所有模块的错误场景

use thiserror::Error;

/// RustSniffer 统一错误类型
#[derive(Error, Debug)]
pub enum SnifferError {
    /// 抓包相关错误
    #[error("抓包错误: {0}")]
    CaptureError(String),

    /// 协议解析错误
    #[error("协议解析错误: {0}")]
    ProtocolError(String),

    /// DPI 检测错误
    #[error("DPI 检测错误: {0}")]
    DpiError(String),

    /// 存储相关错误
    #[error("存储错误: {0}")]
    StorageError(String),

    /// 配置相关错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// I/O 错误
    #[error("I/O 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    /// 配置反序列化错误
    #[error("配置解析错误: {0}")]
    TomlError(#[from] toml::de::Error),

    /// 其他未分类错误
    #[error("未分类错误: {0}")]
    Other(String),
}

/// 为 SnifferError 添加上下文信息
impl SnifferError {
    /// 创建抓包错误
    pub fn capture<S: Into<String>>(msg: S) -> Self {
        SnifferError::CaptureError(msg.into())
    }

    /// 创建协议解析错误
    pub fn protocol<S: Into<String>>(msg: S) -> Self {
        SnifferError::ProtocolError(msg.into())
    }

    /// 创建 DPI 错误
    pub fn dpi<S: Into<String>>(msg: S) -> Self {
        SnifferError::DpiError(msg.into())
    }

    /// 创建存储错误
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        SnifferError::StorageError(msg.into())
    }

    /// 创建配置错误
    pub fn config<S: Into<String>>(msg: S) -> Self {
        SnifferError::ConfigError(msg.into())
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, SnifferError>;
