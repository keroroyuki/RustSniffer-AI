//! 公共类型定义
//!
//! 定义五元组、协议、方向等核心类型

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// 传输层协议枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// TCP 协议
    TCP,
    /// UDP 协议
    UDP,
    /// ICMP 协议
    ICMP,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::TCP => write!(f, "TCP"),
            Protocol::UDP => write!(f, "UDP"),
            Protocol::ICMP => write!(f, "ICMP"),
        }
    }
}

/// 五元组结构体，用于标识网络流
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FiveTuple {
    /// 源 IP 地址
    pub src_ip: IpAddr,
    /// 目的 IP 地址
    pub dst_ip: IpAddr,
    /// 源端口（TCP/UDP）
    pub src_port: u16,
    /// 目的端口（TCP/UDP）
    pub dst_port: u16,
    /// 传输层协议
    pub protocol: Protocol,
}

impl FiveTuple {
    /// 创建新的五元组
    pub fn new(
        src_ip: IpAddr,
        dst_ip: IpAddr,
        src_port: u16,
        dst_port: u16,
        protocol: Protocol,
    ) -> Self {
        Self {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
        }
    }

    /// 反转五元组（交换源和目的）
    pub fn reverse(&self) -> Self {
        Self {
            src_ip: self.dst_ip,
            dst_ip: self.src_ip,
            src_port: self.dst_port,
            dst_port: self.src_port,
            protocol: self.protocol,
        }
    }
}

impl std::fmt::Display for FiveTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.protocol {
            Protocol::TCP | Protocol::UDP => {
                write!(
                    f,
                    "{}:{} -> {}:{} ({})",
                    self.src_ip, self.src_port, self.dst_ip, self.dst_port, self.protocol
                )
            }
            Protocol::ICMP => {
                write!(f, "{} -> {} ({})", self.src_ip, self.dst_ip, self.protocol)
            }
        }
    }
}

/// 数据包方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// 入站流量
    Inbound,
    /// 出站流量
    Outbound,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Inbound => write!(f, "inbound"),
            Direction::Outbound => write!(f, "outbound"),
        }
    }
}

/// 数据包基本信息
#[derive(Debug, Clone)]
pub struct PacketInfo {
    /// 捕获时间戳（Unix 毫秒）
    pub timestamp: i64,
    /// 原始数据包字节
    pub raw_bytes: Vec<u8>,
    /// 网卡标识
    pub interface_id: String,
}

impl PacketInfo {
    /// 创建新的数据包信息
    pub fn new(timestamp: i64, raw_bytes: Vec<u8>, interface_id: String) -> Self {
        Self {
            timestamp,
            raw_bytes,
            interface_id,
        }
    }

    /// 获取数据包长度
    pub fn len(&self) -> usize {
        self.raw_bytes.len()
    }

    /// 检查数据包是否为空
    pub fn is_empty(&self) -> bool {
        self.raw_bytes.is_empty()
    }
}
