//! 命令行参数定义
//!
//! 使用 clap derive 定义 CLI 参数结构

use clap::{Parser, Subcommand};

/// RustSniffer - Rust 智能抓包与流量分析工具
#[derive(Parser, Debug)]
#[command(name = "rustsniffer")]
#[command(author = "RustSniffer Team")]
#[command(version = "0.1.0")]
#[command(about = "Rust 智能抓包与流量分析工具", long_about = None)]
pub struct Cli {
    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 网卡名称（不指定则自动选择第一个）
    #[arg(short, long)]
    pub interface: Option<String>,

    /// BPF 过滤表达式
    #[arg(short, long)]
    pub filter: Option<String>,

    /// 启用混杂模式
    #[arg(short = 'P', long, default_value_t = true)]
    pub promiscuous: bool,

    /// 最大捕获包数（0 表示无限制）
    #[arg(short, long, default_value_t = 0)]
    pub count: u64,

    /// 最大捕获时长（秒，0 表示无限制）
    #[arg(short, long, default_value_t = 0)]
    pub duration: u64,
}

/// 子命令枚举
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 列出可用网卡
    #[command(name = "list-interfaces")]
    ListInterfaces,

    /// 开始抓包
    #[command(name = "capture")]
    Capture {
        /// 网卡名称
        #[arg(short, long)]
        interface: Option<String>,

        /// BPF 过滤表达式
        #[arg(short, long)]
        filter: Option<String>,
    },
}
