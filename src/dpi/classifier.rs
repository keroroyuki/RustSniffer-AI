//! 协议分类器
//!
//! 基于端口和 DPI 结果的应用层协议识别

use crate::common::types::Protocol;
use crate::dpi::engine::DpiResult;
use std::collections::HashMap;

/// 协议分类器
///
/// 结合端口匹配和 DPI 检测结果，确定最终的应用层协议
pub struct ProtocolClassifier {
    /// 端口到协议的映射表
    port_map: HashMap<u16, String>,
}

impl ProtocolClassifier {
    /// 创建新的分类器
    pub fn new() -> Self {
        let mut port_map = HashMap::new();

        // 常见端口映射
        port_map.insert(20, "FTP-DATA".to_string());
        port_map.insert(21, "FTP".to_string());
        port_map.insert(22, "SSH".to_string());
        port_map.insert(23, "TELNET".to_string());
        port_map.insert(25, "SMTP".to_string());
        port_map.insert(53, "DNS".to_string());
        port_map.insert(80, "HTTP".to_string());
        port_map.insert(110, "POP3".to_string());
        port_map.insert(143, "IMAP".to_string());
        port_map.insert(443, "HTTPS".to_string());
        port_map.insert(993, "IMAPS".to_string());
        port_map.insert(995, "POP3S".to_string());
        port_map.insert(3306, "MYSQL".to_string());
        port_map.insert(5432, "POSTGRESQL".to_string());
        port_map.insert(6379, "REDIS".to_string());
        port_map.insert(27017, "MONGODB".to_string());

        Self { port_map }
    }

    /// 根据端口识别协议
    pub fn classify_by_port(&self, port: u16) -> Option<String> {
        self.port_map.get(&port).cloned()
    }

    /// 综合分类：结合端口和 DPI 结果
    ///
    /// 优先级：DPI 结果 > 端口匹配
    pub fn classify(
        &self,
        protocol: Protocol,
        src_port: u16,
        dst_port: u16,
        dpi_result: Option<&DpiResult>,
    ) -> String {
        // 优先使用 DPI 结果
        if let Some(dpi) = dpi_result {
            if dpi.confidence > 0.5 && !dpi.app_protocol.is_empty() && dpi.app_protocol != "Unknown"
            {
                return dpi.app_protocol.clone();
            }
        }

        // 端口匹配
        if let Some(app) = self.classify_by_port(dst_port) {
            return app;
        }
        if let Some(app) = self.classify_by_port(src_port) {
            return app;
        }

        // 基于传输层协议返回默认值
        match protocol {
            Protocol::TCP => "TCP".to_string(),
            Protocol::UDP => "UDP".to_string(),
            Protocol::ICMP => "ICMP".to_string(),
        }
    }
}

impl Default for ProtocolClassifier {
    fn default() -> Self {
        Self::new()
    }
}
