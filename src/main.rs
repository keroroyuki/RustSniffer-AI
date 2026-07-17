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

/// 初始化 Npcap DLL 路径（Windows 专用）
///
/// 在 Windows 系统上，Npcap 的 DLL 文件（wpcap.dll、Packet.dll）通常安装在
/// C:\Windows\System32\Npcap 目录下，但该目录可能不在系统 PATH 中。
/// 此函数检测并添加 Npcap 路径到 PATH 环境变量，确保程序能够找到所需的 DLL。
#[cfg(target_os = "windows")]
fn init_npcap_path() {
    use std::env;
    use std::path::Path;

    // Npcap 常见安装路径
    let npcak_paths = [
        r"C:\Windows\System32\Npcap",
        r"C:\Windows\SysWOW64\Npcap",
    ];

    // 检查是否已存在 wpcap.dll
    for npcak_path in &npcak_paths {
        let wpcap_dll = Path::new(npcak_path).join("wpcap.dll");
        if wpcap_dll.exists() {
            // 检查 PATH 中是否已包含此路径
            if let Ok(current_path) = env::var("PATH") {
                if !current_path.to_lowercase().contains(&npcak_path.to_lowercase()) {
                    // 添加到 PATH 开头
                    let new_path = format!("{};{}", npcak_path, current_path);
                    env::set_var("PATH", &new_path);
                    info!("已自动添加 Npcap 路径到 PATH: {}", npcak_path);
                }
            }
            break;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn init_npcap_path() {
    // 非 Windows 系统无需处理
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 Npcap 路径（Windows 专用）
    init_npcap_path();

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
