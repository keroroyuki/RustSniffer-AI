//! BPF 过滤器处理
//!
//! 提供 BPF 表达式验证和应用功能

use anyhow::{Context, Result};
use pcap::{Capture, Device};

/// 验证 BPF 表达式语法
pub fn validate_bpf(expression: &str) -> Result<()> {
    // 创建一个临时捕获对象来验证 BPF 语法
    let device = Device::lookup()
        .context("无法查找网卡")?
        .ok_or_else(|| anyhow::anyhow!("未找到可用网卡"))?;
    
    let mut cap = Capture::from_device(device)
        .context("无法打开网卡进行 BPF 验证")?
        .open()
        .context("无法激活网卡进行 BPF 验证")?;

    cap.filter(expression, true)
        .context(format!("BPF 表达式语法错误: '{}'", expression))?;

    Ok(())
}

/// 应用 BPF 过滤器到捕获对象
pub fn apply_bpf(capture: &mut Capture<pcap::Active>, expression: &str) -> Result<()> {
    if expression.is_empty() {
        return Ok(());
    }

    capture
        .filter(expression, true)
        .context(format!("应用 BPF 过滤器失败: '{}'", expression))?;

    Ok(())
}

