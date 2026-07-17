//! 协议解析器 Trait 定义
//!
//! 定义协议解析的统一接口和数据结构

use crate::common::types::FiveTuple;
use serde::{Deserialize, Serialize};

/// 解析后的数据包
#[derive(Debug, Clone)]
pub struct ParsedPacket {
    /// 五元组信息
    pub five_tuple: FiveTuple,
    /// 协议元数据
    pub metadata: ProtocolMetadata,
    /// 原始数据包字节
    pub raw_bytes: Vec<u8>,
    /// 时间戳（Unix 毫秒）
    pub timestamp: i64,
}

/// 协议元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    /// Ethernet 类型（0x0800=IPv4, 0x86DD=IPv6）
    pub ethernet_type: Option<u16>,
    /// IP 版本（4 或 6）
    pub ip_version: Option<u8>,
    /// IP TTL 值
    pub ip_ttl: Option<u8>,
    /// 传输层协议名称
    pub transport_protocol: String,
    /// DPI 识别的应用层协议
    pub app_protocol: Option<String>,
    /// 应用层字段（JSON 格式）
    pub app_metadata: Option<serde_json::Value>,
}

impl Default for ProtocolMetadata {
    fn default() -> Self {
        Self {
            ethernet_type: None,
            ip_version: None,
            ip_ttl: None,
            transport_protocol: "Unknown".to_string(),
            app_protocol: None,
            app_metadata: None,
        }
    }
}

/// 协议解析器 Trait
pub trait ProtocolParser {
    /// 解析原始数据包
    fn parse(&self, raw: &[u8]) -> anyhow::Result<ParsedPacket>;
}

/// 协议解析器实现
pub struct DefaultProtocolParser;

impl DefaultProtocolParser {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolParser for DefaultProtocolParser {
    fn parse(&self, raw: &[u8]) -> anyhow::Result<ParsedPacket> {
        // 使用 etherparse 进行协议解析
        use etherparse::SlicedPacket;
        
        let sliced = SlicedPacket::from_ethernet(raw)
            .map_err(|e| anyhow::anyhow!("协议解析失败: {}", e))?;
        
        let mut metadata = ProtocolMetadata::default();
        
        // 解析 Ethernet 层
        if let Some(link) = sliced.link {
            use etherparse::LinkSlice;
            if let LinkSlice::Ethernet2(eth) = link {
                metadata.ethernet_type = Some(eth.ether_type().0);
            }
        }
        
        // 解析 IP 层
        let mut src_ip = std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED);
        let mut dst_ip = std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED);
        
        if let Some(net) = sliced.net {
            use etherparse::NetSlice;
            match net {
                NetSlice::Ipv4(ipv4) => {
                    let header = ipv4.header();
                    metadata.ip_version = Some(4);
                    metadata.ip_ttl = Some(header.ttl());
                    src_ip = std::net::IpAddr::V4(header.source().into());
                    dst_ip = std::net::IpAddr::V4(header.destination().into());
                }
                NetSlice::Ipv6(ipv6) => {
                    let header = ipv6.header();
                    metadata.ip_version = Some(6);
                    metadata.ip_ttl = Some(header.hop_limit());
                    src_ip = std::net::IpAddr::V6(header.source().into());
                    dst_ip = std::net::IpAddr::V6(header.destination().into());
                }
            }
        }
        
        // 解析传输层
        let mut src_port = 0u16;
        let mut dst_port = 0u16;
        let mut protocol = crate::common::types::Protocol::TCP;
        
        if let Some(transport) = sliced.transport {
            use etherparse::TransportSlice;
            match transport {
                TransportSlice::Tcp(tcp) => {
                    let header = tcp.to_header();
                    protocol = crate::common::types::Protocol::TCP;
                    src_port = header.source_port;
                    dst_port = header.destination_port;
                    metadata.transport_protocol = "TCP".to_string();
                }
                TransportSlice::Udp(udp) => {
                    let header = udp.to_header();
                    protocol = crate::common::types::Protocol::UDP;
                    src_port = header.source_port;
                    dst_port = header.destination_port;
                    metadata.transport_protocol = "UDP".to_string();
                }
                TransportSlice::Icmpv4(icmpv4) => {
                    protocol = crate::common::types::Protocol::ICMP;
                    metadata.transport_protocol = "ICMP".to_string();
                    // 解析 ICMP 类型和代码
                    let icmp_data = icmpv4.payload();
                    if icmp_data.len() >= 2 {
                        let icmp_type = icmp_data[0];
                        let icmp_code = icmp_data[1];
                        metadata.app_protocol = Some(format!(
                            "ICMP (type={}, code={})",
                            icmp_type, icmp_code
                        ));
                    }
                }
                TransportSlice::Icmpv6(icmpv6) => {
                    protocol = crate::common::types::Protocol::ICMP;
                    metadata.transport_protocol = "ICMPv6".to_string();
                    // 解析 ICMPv6 类型和代码
                    let icmp_data = icmpv6.payload();
                    if icmp_data.len() >= 2 {
                        let icmp_type = icmp_data[0];
                        let icmp_code = icmp_data[1];
                        metadata.app_protocol = Some(format!(
                            "ICMPv6 (type={}, code={})",
                            icmp_type, icmp_code
                        ));
                    }
                }
            }
        }
        
        let five_tuple = FiveTuple::new(src_ip, dst_ip, src_port, dst_port, protocol);
        
        Ok(ParsedPacket {
            five_tuple,
            metadata,
            raw_bytes: raw.to_vec(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
}

impl Default for DefaultProtocolParser {
    fn default() -> Self {
        Self::new()
    }
}


