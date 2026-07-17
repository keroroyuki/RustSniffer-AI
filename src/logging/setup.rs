//! 日志初始化与轮转
//!
//! 使用 tracing + tracing-subscriber + tracing-appender 实现结构化日志

use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::common::error::{Result, SnifferError};
use crate::common::utils::get_log_dir;
use crate::config::settings::LoggingConfig;

/// 初始化日志系统
///
/// 根据配置设置日志级别、输出格式和文件轮转
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    // 验证日志级别有效性
    let _level = parse_level(&config.level)?;

    // 创建日志目录
    let log_dir = if let Some(ref dir) = config.log_dir {
        std::path::PathBuf::from(dir)
    } else {
        get_log_dir()?
    };

    // 确保日志目录存在
    crate::common::utils::ensure_dir(&log_dir)?;

    // 创建文件追加器（按大小轮转）
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("rustsniffer.log")
        .max_log_files(config.max_log_files as usize)
        .build(&log_dir)
        .map_err(|e| SnifferError::config(format!("无法创建日志文件: {}", e)))?;

    // 构建环境变量过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // 构建并初始化订阅者
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        );

    // 根据配置选择输出格式
    if config.json_format {
        subscriber
            .with(fmt::layer().json())
            .try_init()
            .map_err(|e| SnifferError::config(format!("日志初始化失败: {}", e)))?;
    } else {
        subscriber
            .with(fmt::layer().pretty())
            .try_init()
            .map_err(|e| SnifferError::config(format!("日志初始化失败: {}", e)))?;
    }

    // 记录初始化成功
    tracing::info!(
        level = %config.level,
        log_dir = %log_dir.display(),
        json_format = config.json_format,
        "日志系统初始化成功"
    );

    Ok(())
}

/// 解析日志级别字符串
fn parse_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(SnifferError::config(format!(
            "无效的日志级别: {}，有效值为: trace, debug, info, warn, error",
            level
        ))),
    }
}
