//! ICMP 解析
//!
//! 实现 ICMP 协议解析，提取类型和代码信息

/// ICMP 类型描述
pub fn icmp_type_to_string(icmp_type: u8) -> &'static str {
    match icmp_type {
        0 => "Echo Reply",
        3 => "Destination Unreachable",
        8 => "Echo Request",
        11 => "Time Exceeded",
        _ => "Unknown",
    }
}

/// 解析 ICMP 数据包
pub fn parse_icmp(data: &[u8]) -> String {
    if data.len() < 4 {
        return "ICMP (truncated)".to_string();
    }

    let icmp_type = data[0];
    let icmp_code = data[1];

    format!(
        "ICMP {} (type={}, code={})",
        icmp_type_to_string(icmp_type),
        icmp_type,
        icmp_code
    )
}
