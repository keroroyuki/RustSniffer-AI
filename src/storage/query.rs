//! 查询接口
//!
//! 提供元数据查询功能，支持按时间、IP、协议等条件检索

use crate::common::error::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// 查询条件
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// 起始时间戳（毫秒）
    pub start_time: Option<i64>,
    /// 结束时间戳（毫秒）
    pub end_time: Option<i64>,
    /// 源 IP 地址
    pub src_ip: Option<String>,
    /// 目的 IP 地址
    pub dst_ip: Option<String>,
    /// 源端口
    pub src_port: Option<u16>,
    /// 目的端口
    pub dst_port: Option<u16>,
    /// 协议类型
    pub protocol: Option<String>,
    /// 应用层协议
    pub app_protocol: Option<String>,
    /// 返回条数限制
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct PacketRecord {
    /// 记录 ID
    pub id: i64,
    /// 时间戳
    pub timestamp: i64,
    /// 源 IP
    pub src_ip: String,
    /// 目的 IP
    pub dst_ip: String,
    /// 源端口
    pub src_port: u16,
    /// 目的端口
    pub dst_port: u16,
    /// 协议
    pub protocol: String,
    /// 传输层协议
    pub transport_protocol: String,
    /// 应用层协议
    pub app_protocol: Option<String>,
    /// 数据包大小
    pub packet_size: usize,
    /// JA3 哈希
    pub ja3_hash: Option<String>,
    /// JA3S 哈希
    pub ja3s_hash: Option<String>,
    /// 原始包路径
    pub raw_packet_path: Option<String>,
}

/// 查询管理器
pub struct QueryManager {
    /// SQLite 连接
    conn: Arc<Mutex<Connection>>,
}

impl QueryManager {
    /// 创建新的查询管理器
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 执行查询
    pub fn query(&self, filter: &QueryFilter) -> Result<Vec<PacketRecord>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from("SELECT id, timestamp, src_ip, dst_ip, src_port, dst_port, protocol, transport_protocol, app_protocol, packet_size, ja3_hash, ja3s_hash, raw_packet_path FROM packet_metadata WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        // 构建 WHERE 子句
        if let Some(start) = filter.start_time {
            sql.push_str(" AND timestamp >= ?");
            param_values.push(Box::new(start));
        }
        if let Some(end) = filter.end_time {
            sql.push_str(" AND timestamp <= ?");
            param_values.push(Box::new(end));
        }
        if let Some(ref src_ip) = filter.src_ip {
            sql.push_str(" AND src_ip = ?");
            param_values.push(Box::new(src_ip.clone()));
        }
        if let Some(ref dst_ip) = filter.dst_ip {
            sql.push_str(" AND dst_ip = ?");
            param_values.push(Box::new(dst_ip.clone()));
        }
        if let Some(src_port) = filter.src_port {
            sql.push_str(" AND src_port = ?");
            param_values.push(Box::new(src_port));
        }
        if let Some(dst_port) = filter.dst_port {
            sql.push_str(" AND dst_port = ?");
            param_values.push(Box::new(dst_port));
        }
        if let Some(ref protocol) = filter.protocol {
            sql.push_str(" AND protocol = ?");
            param_values.push(Box::new(protocol.clone()));
        }
        if let Some(ref app_protocol) = filter.app_protocol {
            sql.push_str(" AND app_protocol = ?");
            param_values.push(Box::new(app_protocol.clone()));
        }

        // 排序
        sql.push_str(" ORDER BY timestamp DESC");

        // 限制
        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // 执行查询
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

        let records = stmt
            .query_map(params.as_slice(), |row| {
                Ok(PacketRecord {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    src_ip: row.get(2)?,
                    dst_ip: row.get(3)?,
                    src_port: row.get(4)?,
                    dst_port: row.get(5)?,
                    protocol: row.get(6)?,
                    transport_protocol: row.get(7)?,
                    app_protocol: row.get(8)?,
                    packet_size: row.get(9)?,
                    ja3_hash: row.get(10)?,
                    ja3s_hash: row.get(11)?,
                    raw_packet_path: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 按 IP 统计流量
    pub fn stats_by_ip(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT src_ip, COUNT(*) as count FROM packet_metadata GROUP BY src_ip ORDER BY count DESC LIMIT 20",
        )?;

        let results = stmt
            .query_map([], |row| {
                let ip: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((ip, count))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// 按协议统计流量
    pub fn stats_by_protocol(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT app_protocol, COUNT(*) as count FROM packet_metadata GROUP BY app_protocol ORDER BY count DESC LIMIT 20",
        )?;

        let results = stmt
            .query_map([], |row| {
                let protocol: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((protocol, count))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }
}
