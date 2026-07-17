//! RustSniffer - Rust 智能抓包与流量分析工具
//!
//! 程序入口

use anyhow::Result;
use clap::Parser;
use tracing::info;

use rustsniffer::cli::args::{Cli, Commands};
use rustsniffer::cli::commands::{cmd_capture, cmd_list_interfaces};
use rustsniffer::config::loader;
use rustsniffer::logging::setup::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = loader::load()?;

    // 初始化日志
    init_logging(&config.logging)?;

    info!("RustSniffer v0.1.0 启动");

    // 解析命令行参数
    let cli = Cli::parse();

    // 处理命令
    match cli.command {
        Some(Commands::ListInterfaces) => {
            cmd_list_interfaces()?;
        }
        Some(Commands::Capture {
            interface,
            filter,
        }) => {
            // 命令行参数优先，否则使用配置文件
            let iface = interface.or(config.capture.interface);
            let flt = filter.or(config.capture.filter);
            let promisc = config.capture.promiscuous;
            let count = if config.capture.count > 0 {
                config.capture.count
            } else {
                cli.count
            };
            let duration = if config.capture.duration > 0 {
                config.capture.duration
            } else {
                cli.duration
            };

            cmd_capture(iface, flt, promisc, count, duration).await?;
        }
        None => {
            // 默认行为：直接抓包
            let iface = cli.interface.or(config.capture.interface);
            let flt = cli.filter.or(config.capture.filter);
            let promisc = config.capture.promiscuous;
            let count = if config.capture.count > 0 {
                config.capture.count
            } else {
                cli.count
            };
            let duration = if config.capture.duration > 0 {
                config.capture.duration
            } else {
                cli.duration
            };

            cmd_capture(iface, flt, promisc, count, duration).await?;
        }
    }

    info!("RustSniffer 正常退出");
    Ok(())
}
