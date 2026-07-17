//! TLS JA3/JA3S 指纹提取
//!
//! 实现 TLS Client Hello 解析和 JA3/JA3S 指纹计算

use md5::{Md5, Digest};

/// JA3 指纹信息
#[derive(Debug, Clone)]
pub struct Ja3Fingerprint {
    /// JA3 原始字符串
    pub ja3_raw: String,
    /// JA3 MD5 哈希
    pub ja3_hash: String,
}

/// JA3S 指纹信息（Server Hello）
#[derive(Debug, Clone)]
pub struct Ja3sFingerprint {
    /// JA3S 原始字符串
    pub ja3s_raw: String,
    /// JA3S MD5 哈希
    pub ja3s_hash: String,
}

/// TLS 版本映射
fn tls_version_to_string(version: u16) -> String {
    match version {
        0x0300 => "3.0",
        0x0301 => "3.1",
        0x0302 => "3.2",
        0x0303 => "3.3",
        0x0304 => "1.3",
        _ => "unknown",
    }.to_string()
}

/// 解析 TLS Client Hello 并提取 JA3 指纹
pub fn extract_ja3(data: &[u8]) -> Option<Ja3Fingerprint> {
    // 简化的 TLS Client Hello 解析
    // 实际实现需要完整的 TLS 解析器
    
    if data.len() < 43 {
        return None;
    }
    
    // 检查是否是 TLS Record
    if data[0] != 0x16 { // Handshake
        return None;
    }
    
    // 跳过 Record Header (5 bytes)
    let mut pos = 5;
    
    // 检查 Handshake Type
    if data[pos] != 0x01 { // Client Hello
        return None;
    }
    pos += 1;
    
    // 跳过 Length (3 bytes)
    pos += 3;
    
    // 提取 Version
    if pos + 2 > data.len() {
        return None;
    }
    let version = u16::from_be_bytes([data[pos], data[pos + 1]]);
    let version_str = tls_version_to_string(version);
    pos += 2;
    
    // 跳过 Random (32 bytes)
    pos += 32;
    
    // 跳过 Session ID
    if pos >= data.len() {
        return None;
    }
    let session_id_len = data[pos] as usize;
    pos += 1 + session_id_len;
    
    // 提取 Cipher Suites
    if pos + 2 > data.len() {
        return None;
    }
    let cipher_suites_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    
    let mut cipher_suites = Vec::new();
    for _ in 0..cipher_suites_len / 2 {
        if pos + 2 > data.len() {
            break;
        }
        let suite = u16::from_be_bytes([data[pos], data[pos + 1]]);
        cipher_suites.push(suite.to_string());
        pos += 2;
    }
    
    // 跳过 Compression Methods
    if pos >= data.len() {
        return None;
    }
    let compression_len = data[pos] as usize;
    pos += 1 + compression_len;
    
    // 提取 Extensions
    let mut extensions = Vec::new();
    if pos + 2 <= data.len() {
        let extensions_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        
        let end_pos = std::cmp::min(pos + extensions_len, data.len());
        while pos + 4 <= end_pos {
            let ext_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let ext_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            extensions.push(ext_type.to_string());
            pos += 4 + ext_len;
        }
    }
    
    // 构建 JA3 字符串
    let ja3_raw = format!(
        "{},{},{},{}",
        version_str,
        cipher_suites.join("-"),
        extensions.join("-"),
        "" // Elliptic curves 和 EC point formats 需要更复杂的解析
    );
    
    // 计算 MD5 哈希
    let mut hasher = Md5::new();
    hasher.update(ja3_raw.as_bytes());
    let ja3_hash = format!("{:x}", hasher.finalize());
    
    Some(Ja3Fingerprint {
        ja3_raw,
        ja3_hash,
    })
}

/// 解析 TLS Server Hello 并提取 JA3S 指纹
pub fn extract_ja3s(data: &[u8]) -> Option<Ja3sFingerprint> {
    // 简化的 TLS Server Hello 解析
    
    if data.len() < 43 {
        return None;
    }
    
    // 检查是否是 TLS Record
    if data[0] != 0x16 { // Handshake
        return None;
    }
    
    // 跳过 Record Header (5 bytes)
    let mut pos = 5;
    
    // 检查 Handshake Type
    if data[pos] != 0x02 { // Server Hello
        return None;
    }
    pos += 1;
    
    // 跳过 Length (3 bytes)
    pos += 3;
    
    // 提取 Version
    if pos + 2 > data.len() {
        return None;
    }
    let version = u16::from_be_bytes([data[pos], data[pos + 1]]);
    let version_str = tls_version_to_string(version);
    pos += 2;
    
    // 跳过 Random (32 bytes)
    pos += 32;
    
    // 跳过 Session ID
    if pos >= data.len() {
        return None;
    }
    let session_id_len = data[pos] as usize;
    pos += 1 + session_id_len;
    
    // 提取 Cipher Suite
    if pos + 2 > data.len() {
        return None;
    }
    let cipher_suite = u16::from_be_bytes([data[pos], data[pos + 1]]);
    pos += 2;
    
    // 跳过 Compression Method
    pos += 1;
    
    // 提取 Extensions
    let mut extensions = Vec::new();
    if pos + 2 <= data.len() {
        let extensions_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        
        let end_pos = std::cmp::min(pos + extensions_len, data.len());
        while pos + 4 <= end_pos {
            let ext_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let ext_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            extensions.push(ext_type.to_string());
            pos += 4 + ext_len;
        }
    }
    
    // 构建 JA3S 字符串
    let ja3s_raw = format!(
        "{},{},{}",
        version_str,
        cipher_suite,
        extensions.join("-")
    );
    
    // 计算 MD5 哈希
    let mut hasher = Md5::new();
    hasher.update(ja3s_raw.as_bytes());
    let ja3s_hash = format!("{:x}", hasher.finalize());
    
    Some(Ja3sFingerprint {
        ja3s_raw,
        ja3s_hash,
    })
}

