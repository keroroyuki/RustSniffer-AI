//! 配置结构定义
//!
//! 定义 RustSniffer 的所有配置项

use serde::{Deserialize, Serialize};

/// 主配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// 抓包配置
    #[serde(default)]
    pub capture: CaptureConfig,
    /// 存储配置
    #[serde(default)]
    pub storage: StorageConfig,
    /// 日志配置
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// 抓包配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// 默认网卡名称（None 表示自动选择第一个）
    pub interface: Option<String>,
    /// 默认 BPF 过滤器
    pub filter: Option<String>,
    /// 是否启用混杂模式
    #[serde(default = "default_true")]
    pub promiscuous: bool,
    /// 最大捕获包数（0 表示无限制）
    #[serde(default)]
    pub count: u64,
    /// 最大捕获时长（秒，0 表示无限制）
    #[serde(default)]
    pub duration: u64,
    /// 抓包缓冲区大小（字节）
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// 流量采样阈值（每秒包数，超过此值启用采样）
    #[serde(default = "default_sampling_threshold")]
    pub sampling_threshold: u64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            interface: None,
            filter: None,
            promiscuous: true,
            count: 0,
            duration: 0,
            buffer_size: default_buffer_size(),
            sampling_threshold: default_sampling_threshold(),
        }
    }
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 数据库路径（默认 ~/.rustsniffer/metadata.db）
    pub db_path: Option<String>,
    /// PCAPNG 文件存储目录（默认 ~/.rustsniffer/pcap/）
    pub pcap_dir: Option<String>,
    /// 数据保留天数
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    /// 批量插入大小
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// 批量插入间隔（毫秒）
    #[serde(default = "default_batch_interval_ms")]
    pub batch_interval_ms: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: None,
            pcap_dir: None,
            retention_days: default_retention_days(),
            batch_size: default_batch_size(),
            batch_interval_ms: default_batch_interval_ms(),
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别（trace, debug, info, warn, error）
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志目录（默认 ~/.rustsniffer/logs/）
    pub log_dir: Option<String>,
    /// 单个日志文件最大大小（MB）
    #[serde(default = "default_max_log_size_mb")]
    pub max_log_size_mb: u64,
    /// 保留的日志文件数量
    #[serde(default = "default_max_log_files")]
    pub max_log_files: u32,
    /// 是否输出 JSON 格式
    #[serde(default)]
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            log_dir: None,
            max_log_size_mb: default_max_log_size_mb(),
            max_log_files: default_max_log_files(),
            json_format: false,
        }
    }
}

// 默认值函数
fn default_true() -> bool {
    true
}

fn default_buffer_size() -> usize {
    2 * 1024 * 1024 // 2MB
}

fn default_sampling_threshold() -> u64 {
    100_000 // 每秒 10 万包
}

fn default_retention_days() -> u32 {
    30
}

fn default_batch_size() -> usize {
    1000
}

fn default_batch_interval_ms() -> u64 {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_size_mb() -> u64 {
    100
}

fn default_max_log_files() -> u32 {
    5
}
