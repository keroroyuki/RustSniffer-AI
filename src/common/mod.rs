//! 公共模块
//!
//! 提供公共类型、错误处理和工具函数

pub mod error;
pub mod types;
pub mod utils;

// 重新导出常用类型，方便其他模块使用
pub use error::{Result, SnifferError};
pub use types::{Direction, FiveTuple, PacketInfo, Protocol};
