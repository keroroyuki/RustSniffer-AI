//! 分层配置加载
//!
//! 实现配置加载优先级：默认值 → 配置文件 → 环境变量 → 命令行参数

use std::env;
use std::fs;

use crate::common::error::{Result, SnifferError};
use crate::common::utils::{ensure_data_dirs, get_config_path};
use crate::config::settings::Settings;

/// 加载配置
///
/// 按照优先级加载配置：默认值 → 配置文件 → 环境变量
pub fn load() -> Result<Settings> {
    // 确保数据目录存在
    ensure_data_dirs()?;

    // 从默认值开始
    let mut settings = Settings::default();

    // 尝试从配置文件加载
    let config_path = get_config_path()?;
    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path).map_err(|e| {
            SnifferError::config(format!("无法读取配置文件 {}: {}", config_path.display(), e))
        })?;

        let file_settings: Settings = toml::from_str(&config_content).map_err(|e| {
            SnifferError::config(format!("配置文件解析失败: {}", e))
        })?;

        // 合并文件配置
        settings = merge_settings(settings, file_settings);
    }

    // 从环境变量覆盖（前缀 RUSTSNIFFER_）
    settings = apply_env_overrides(settings);

    Ok(settings)
}

/// 重新加载配置（用于 SIGHUP 信号处理）
pub fn reload() -> Result<Settings> {
    load()
}

/// 合并配置（后加载的配置覆盖先加载的）
fn merge_settings(base: Settings, override_settings: Settings) -> Settings {
    Settings {
        capture: merge_capture(base.capture, override_settings.capture),
        storage: merge_storage(base.storage, override_settings.storage),
        logging: merge_logging(base.logging, override_settings.logging),
    }
}

fn merge_capture(
    base: crate::config::settings::CaptureConfig,
    override_cfg: crate::config::settings::CaptureConfig,
) -> crate::config::settings::CaptureConfig {
    crate::config::settings::CaptureConfig {
        interface: override_cfg.interface.or(base.interface),
        filter: override_cfg.filter.or(base.filter),
        promiscuous: override_cfg.promiscuous,
        count: if override_cfg.count > 0 {
            override_cfg.count
        } else {
            base.count
        },
        duration: if override_cfg.duration > 0 {
            override_cfg.duration
        } else {
            base.duration
        },
        buffer_size: override_cfg.buffer_size,
        sampling_threshold: override_cfg.sampling_threshold,
    }
}

fn merge_storage(
    base: crate::config::settings::StorageConfig,
    override_cfg: crate::config::settings::StorageConfig,
) -> crate::config::settings::StorageConfig {
    crate::config::settings::StorageConfig {
        db_path: override_cfg.db_path.or(base.db_path),
        pcap_dir: override_cfg.pcap_dir.or(base.pcap_dir),
        retention_days: override_cfg.retention_days,
        batch_size: override_cfg.batch_size,
        batch_interval_ms: override_cfg.batch_interval_ms,
    }
}

fn merge_logging(
    base: crate::config::settings::LoggingConfig,
    override_cfg: crate::config::settings::LoggingConfig,
) -> crate::config::settings::LoggingConfig {
    crate::config::settings::LoggingConfig {
        level: if override_cfg.level != "info" {
            override_cfg.level
        } else {
            base.level
        },
        log_dir: override_cfg.log_dir.or(base.log_dir),
        max_log_size_mb: override_cfg.max_log_size_mb,
        max_log_files: override_cfg.max_log_files,
        json_format: override_cfg.json_format || base.json_format,
    }
}

/// 应用环境变量覆盖
fn apply_env_overrides(mut settings: Settings) -> Settings {
    // RUSTSNIFFER_LOG_LEVEL
    if let Ok(level) = env::var("RUSTSNIFFER_LOG_LEVEL") {
        settings.logging.level = level;
    }

    // RUSTSNIFFER_INTERFACE
    if let Ok(interface) = env::var("RUSTSNIFFER_INTERFACE") {
        settings.capture.interface = Some(interface);
    }

    // RUSTSNIFFER_FILTER
    if let Ok(filter) = env::var("RUSTSNIFFER_FILTER") {
        settings.capture.filter = Some(filter);
    }

    // RUSTSNIFFER_DB_PATH
    if let Ok(db_path) = env::var("RUSTSNIFFER_DB_PATH") {
        settings.storage.db_path = Some(db_path);
    }

    // RUSTSNIFFER_PCAP_DIR
    if let Ok(pcap_dir) = env::var("RUSTSNIFFER_PCAP_DIR") {
        settings.storage.pcap_dir = Some(pcap_dir);
    }

    // RUSTSNIFFER_LOG_DIR
    if let Ok(log_dir) = env::var("RUSTSNIFFER_LOG_DIR") {
        settings.logging.log_dir = Some(log_dir);
    }

    settings
}
