//! 网卡枚举与选择
//!
//! 提供网卡列表查询和自动选择功能

use anyhow::{Context, Result};
use pcap::Device;

/// 网卡信息
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    /// 网卡名称
    pub name: String,
    /// 网卡描述
    pub description: String,
    /// IP 地址列表
    pub addresses: Vec<String>,
}

/// 列出所有可用网卡
pub fn list_interfaces() -> Result<Vec<InterfaceInfo>> {
    let devices = Device::list()
        .context("无法枚举网卡，请检查系统权限")?;

    let interfaces = devices
        .into_iter()
        .map(|dev| {
            let addresses = dev.addresses
                .iter()
                .map(|addr| addr.addr.to_string())
                .collect();

            InterfaceInfo {
                name: dev.name,
                description: dev.desc.unwrap_or_else(|| "未知网卡".to_string()),
                addresses,
            }
        })
        .collect();

    Ok(interfaces)
}

/// 选择网卡（自动选择或按名称匹配）
pub fn select_interface(name: Option<&str>) -> Result<String> {
    match name {
        Some(n) => {
            // 验证网卡是否存在
            let interfaces = list_interfaces()?;
            if interfaces.iter().any(|iface| iface.name == n) {
                Ok(n.to_string())
            } else {
                anyhow::bail!("网卡 '{}' 不存在，请使用 list-interfaces 查看可用网卡", n)
            }
        }
        None => {
            // 自动选择第一个非回环网卡
            let interfaces = list_interfaces()?;
            interfaces
                .into_iter()
                .find(|iface| !iface.name.contains("lo") && !iface.description.contains("Loopback"))
                .map(|iface| iface.name)
                .context("未找到可用的网卡，请手动指定网卡名称")
        }
    }
}

