//! 工具函数
//!
//! 提供数据目录路径获取、目录创建等通用工具函数

use std::fs;
use std::path::PathBuf;

use crate::common::error::{Result, SnifferError};

/// 获取 RustSniffer 数据根目录 `~/.rustsniffer/`
pub fn get_data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| SnifferError::config("无法获取用户主目录"))?;
    Ok(home.join(".rustsniffer"))
}

/// 获取 PCAPNG 文件存储目录 `~/.rustsniffer/pcap/`
pub fn get_pcap_dir() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("pcap"))
}

/// 获取日志文件存储目录 `~/.rustsniffer/logs/`
pub fn get_log_dir() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("logs"))
}

/// 获取配置文件路径 `~/.rustsniffer/config.toml`
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("config.toml"))
}

/// 获取数据库文件路径 `~/.rustsniffer/metadata.db`
pub fn get_db_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("metadata.db"))
}

/// 确保指定目录存在，不存在则创建
pub fn ensure_dir(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            SnifferError::config(format!("无法创建目录 {}: {}", path.display(), e))
        })?;
    }
    Ok(())
}

/// 确保 RustSniffer 所有必要的目录存在
pub fn ensure_data_dirs() -> Result<()> {
    ensure_dir(&get_data_dir()?)?;
    ensure_dir(&get_pcap_dir()?)?;
    ensure_dir(&get_log_dir()?)?;
    Ok(())
}
