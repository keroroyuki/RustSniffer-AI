//! 集成测试
//!
//! 测试协议解析和元数据存储功能

use rustsniffer::protocol::parser::{DefaultProtocolParser, ProtocolParser};
use rustsniffer::storage::metadata::MetadataStore;
use tempfile::TempDir;

#[test]
fn test_protocol_parser() {
    let parser = DefaultProtocolParser::new();
    
    // 创建一个简单的以太网帧（IPv4 + TCP）
    let raw_packet = vec![
        // Ethernet header (14 bytes)
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, // dst mac
        0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, // src mac
        0x08, 0x00, // ether type (IPv4)
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28, // version, IHL, total length
        0x00, 0x01, 0x00, 0x00, // identification, flags, fragment offset
        0x40, 0x06, 0x00, 0x00, // TTL, protocol (TCP), checksum
        0xc0, 0xa8, 0x01, 0x01, // src ip (192.168.1.1)
        0xc0, 0xa8, 0x01, 0x02, // dst ip (192.168.1.2)
        // TCP header (20 bytes)
        0x00, 0x50, 0x1f, 0x90, // src port (80), dst port (8080)
        0x00, 0x00, 0x00, 0x01, // sequence number
        0x00, 0x00, 0x00, 0x00, // acknowledgment number
        0x50, 0x02, 0x20, 0x00, // data offset, flags, window size
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ];
    
    let result = parser.parse(&raw_packet);
    assert!(result.is_ok(), "协议解析应该成功");
    
    let parsed = result.unwrap();
    assert_eq!(parsed.five_tuple.src_port, 80);
    assert_eq!(parsed.five_tuple.dst_port, 8080);
    assert_eq!(parsed.metadata.transport_protocol, "TCP");
    assert_eq!(parsed.metadata.ip_version, Some(4));
}

#[test]
fn test_metadata_store() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let store = MetadataStore::new(&db_path).unwrap();
    
    // 创建测试数据包
    let parser = DefaultProtocolParser::new();
    let raw_packet = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
        0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0x08, 0x00,
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x01, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        0xc0, 0xa8, 0x01, 0x01,
        0xc0, 0xa8, 0x01, 0x02,
        0x00, 0x50, 0x1f, 0x90,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x02, 0x20, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];
    
    let parsed = parser.parse(&raw_packet).unwrap();
    
    // 插入元数据
    let id = store.insert(&parsed, None).unwrap();
    assert!(id > 0, "插入应该返回有效的 ID");
    
    // 获取统计信息
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_packets, 1);
    assert_eq!(stats.unique_src_ips, 1);
    assert_eq!(stats.unique_dst_ips, 1);
}

#[test]
fn test_metadata_store_batch_insert() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_batch.db");
    
    let store = MetadataStore::new(&db_path).unwrap();
    
    // 创建多个测试数据包
    let parser = DefaultProtocolParser::new();
    let mut packets = Vec::new();
    
    for i in 0..5 {
        let raw_packet = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
            0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
            0x08, 0x00,
            0x45, 0x00, 0x00, 0x28,
            0x00, 0x01, 0x00, 0x00,
            0x40, 0x06, 0x00, 0x00,
            0xc0, 0xa8, 0x01, (i + 1) as u8,
            0xc0, 0xa8, 0x01, (i + 2) as u8,
            0x00, 0x50, 0x1f, 0x90,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00,
            0x50, 0x02, 0x20, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        
        let parsed = parser.parse(&raw_packet).unwrap();
        packets.push(parsed);
    }
    
    // 批量插入
    let count = store.insert_batch(&packets).unwrap();
    assert_eq!(count, 5, "应该插入 5 条记录");
    
    // 验证统计信息
    let stats = store.stats().unwrap();
    assert_eq!(stats.total_packets, 5);
    assert_eq!(stats.unique_src_ips, 5);
    assert_eq!(stats.unique_dst_ips, 5);
}
