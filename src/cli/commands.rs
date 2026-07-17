//! 命令处理逻辑
//!
//! 实现 CLI 命令的具体执行逻辑

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use crate::capture::interface::{list_interfaces, select_interface};
use crate::capture::sniffer::{CaptureConfig, Sniffer};
use crate::common::types::PacketInfo;

/// 列出所有可用网卡
pub fn cmd_list_interfaces() -> Result<()> {
    let interfaces = list_interfaces()?;

    if interfaces.is_empty() {
        println!("未找到可用网卡");
        return Ok(());
    }

    println!("可用网卡列表：");
    println!("{:-<80}", "");
    for (idx, iface) in interfaces.iter().enumerate() {
        println!("{}. {}", idx + 1, iface.name);
        println!("   描述: {}", iface.description);
        if !iface.addresses.is_empty() {
            println!("   地址: {}", iface.addresses.join(", "));
        }
        println!();
    }

    Ok(())
}

/// 运行抓包
pub async fn cmd_capture(
    interface: Option<String>,
    filter: Option<String>,
    promiscuous: bool,
    count: u64,
    duration: u64,
) -> Result<()> {
    // 选择网卡
    let iface_name = select_interface(interface.as_deref())?;
    info!("选择网卡: {}", iface_name);

    // 创建抓包配置
    let config = CaptureConfig {
        interface: iface_name,
        filter,
        promiscuous,
        count,
        duration,
        buffer_size: 2 * 1024 * 1024, // 2MB
    };

    // 创建通道
    let (tx, mut rx) = mpsc::channel::<PacketInfo>(10000);

    // 启动抓包任务
    let capture_handle = tokio::spawn(async move {
        let mut sniffer = Sniffer::new(config);
        if let Err(e) = sniffer.start(tx).await {
            eprintln!("抓包错误: {}", e);
        }
    });

    // 接收并打印数据包
    println!("开始抓包，按 Ctrl+C 停止...");
    println!("{:-<80}", "");

    let mut packet_count = 0u64;

    while let Some(packet) = rx.recv().await {
        packet_count += 1;

        // 简单打印数据包信息（后续会集成协议解析）
        println!(
            "[{}] {} 字节 - {}",
            packet.timestamp,
            packet.raw_bytes.len(),
            packet.interface_id
        );

        // 每 100 个包打印一次统计
        if packet_count.is_multiple_of(100) {
            println!("已捕获 {} 个数据包", packet_count);
        }
    }

    // 等待抓包任务完成
    capture_handle.await?;

    println!("\n抓包完成，共捕获 {} 个数据包", packet_count);

    Ok(())
}

