//! 元数据存储（SQLite）
//!
//! 使用 SQLite 存储数据包元数据，支持快速查询和检索

use crate::common::error::Result;
use crate::protocol::parser::ParsedPacket;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 元数据存储管理器
pub struct MetadataStore {
    /// SQLite 连接
    conn: Arc<Mutex<Connection>>,
}

impl MetadataStore {
    /// 创建新的元数据存储
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式以提高并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // 创建元数据表
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS packet_metadata (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                src_ip TEXT NOT NULL,
                dst_ip TEXT NOT NULL,
                src_port INTEGER,
                dst_port INTEGER,
                protocol TEXT NOT NULL,
                transport_protocol TEXT NOT NULL,
                app_protocol TEXT,
                ip_version INTEGER,
                ip_ttl INTEGER,
                ethernet_type INTEGER,
                packet_size INTEGER NOT NULL,
                ja3_hash TEXT,
                ja3s_hash TEXT,
                raw_packet_path TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            -- 创建索引以加速查询
            CREATE INDEX IF NOT EXISTS idx_timestamp ON packet_metadata(timestamp);
            CREATE INDEX IF NOT EXISTS idx_src_ip ON packet_metadata(src_ip);
            CREATE INDEX IF NOT EXISTS idx_dst_ip ON packet_metadata(dst_ip);
            CREATE INDEX IF NOT EXISTS idx_protocol ON packet_metadata(protocol);
            CREATE INDEX IF NOT EXISTS idx_app_protocol ON packet_metadata(app_protocol);
            CREATE INDEX IF NOT EXISTS idx_src_dst ON packet_metadata(src_ip, dst_ip);
            "#,
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 插入数据包元数据
    pub fn insert(&self, packet: &ParsedPacket, raw_packet_path: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        // 提取 JA3 哈希（如果有）
        let ja3_hash = packet.metadata.app_metadata.as_ref().and_then(|m| {
            m.get("ja3_hash").and_then(|v| v.as_str()).map(|s| s.to_string())
        });

        let ja3s_hash = packet.metadata.app_metadata.as_ref().and_then(|m| {
            m.get("ja3s_hash").and_then(|v| v.as_str()).map(|s| s.to_string())
        });

        conn.execute(
            r#"
            INSERT INTO packet_metadata (
                timestamp, src_ip, dst_ip, src_port, dst_port,
                protocol, transport_protocol, app_protocol,
                ip_version, ip_ttl, ethernet_type, packet_size,
                ja3_hash, ja3s_hash, raw_packet_path
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                packet.timestamp,
                packet.five_tuple.src_ip.to_string(),
                packet.five_tuple.dst_ip.to_string(),
                packet.five_tuple.src_port,
                packet.five_tuple.dst_port,
                packet.five_tuple.protocol.to_string(),
                packet.metadata.transport_protocol,
                packet.metadata.app_protocol,
                packet.metadata.ip_version,
                packet.metadata.ip_ttl,
                packet.metadata.ethernet_type,
                packet.raw_bytes.len(),
                ja3_hash,
                ja3s_hash,
                raw_packet_path,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 批量插入元数据
    pub fn insert_batch(&self, packets: &[ParsedPacket]) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut count = 0;

        for packet in packets {
            conn.execute(
                r#"
                INSERT INTO packet_metadata (
                    timestamp, src_ip, dst_ip, src_port, dst_port,
                    protocol, transport_protocol, app_protocol,
                    ip_version, ip_ttl, ethernet_type, packet_size
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    packet.timestamp,
                    packet.five_tuple.src_ip.to_string(),
                    packet.five_tuple.dst_ip.to_string(),
                    packet.five_tuple.src_port,
                    packet.five_tuple.dst_port,
                    packet.five_tuple.protocol.to_string(),
                    packet.metadata.transport_protocol,
                    packet.metadata.app_protocol,
                    packet.metadata.ip_version,
                    packet.metadata.ip_ttl,
                    packet.metadata.ethernet_type,
                    packet.raw_bytes.len(),
                ],
            )?;
            count += 1;
        }

        Ok(count)
    }

    /// 获取元数据统计信息
    pub fn stats(&self) -> Result<MetadataStats> {
        let conn = self.conn.lock().unwrap();

        let total_packets: i64 = conn.query_row(
            "SELECT COUNT(*) FROM packet_metadata",
            [],
            |row| row.get(0),
        )?;

        let unique_src_ips: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT src_ip) FROM packet_metadata",
            [],
            |row| row.get(0),
        )?;

        let unique_dst_ips: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT dst_ip) FROM packet_metadata",
            [],
            |row| row.get(0),
        )?;

        let min_timestamp: i64 = conn.query_row(
            "SELECT MIN(timestamp) FROM packet_metadata",
            [],
            |row| row.get(0),
        )?;

        let max_timestamp: i64 = conn.query_row(
            "SELECT MAX(timestamp) FROM packet_metadata",
            [],
            |row| row.get(0),
        )?;

        Ok(MetadataStats {
            total_packets,
            unique_src_ips,
            unique_dst_ips,
            min_timestamp,
            max_timestamp,
        })
    }

    /// 获取数据库连接（用于查询）
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}

/// 元数据统计信息
#[derive(Debug, Clone)]
pub struct MetadataStats {
    /// 总数据包数
    pub total_packets: i64,
    /// 唯一源 IP 数
    pub unique_src_ips: i64,
    /// 唯一目的 IP 数
    pub unique_dst_ips: i64,
    /// 最早时间戳
    pub min_timestamp: i64,
    /// 最晚时间戳
    pub max_timestamp: i64,
}
