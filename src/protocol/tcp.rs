//! TCP 解析与流重组
//!
//! 实现 TCP 协议解析和流重组功能

use crate::common::types::FiveTuple;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// TCP 流重组状态
#[derive(Debug, Clone, PartialEq)]
pub enum ReassemblyStatus {
    /// 重组中
    InProgress,
    /// 重组成功
    Completed,
    /// 重组失败（缺失 N 个包）
    Failed(u32),
}

/// TCP 流结构体
#[derive(Debug, Clone)]
pub struct TcpStream {
    /// 五元组标识
    pub five_tuple: FiveTuple,
    /// 序列号到数据的映射（用于乱序包重组）
    pub segments: BTreeMap<u32, Vec<u8>>,
    /// 期望的下一个序列号
    pub next_seq: u32,
    /// 重组状态
    pub status: ReassemblyStatus,
    /// 缺失包计数
    pub missing_count: u32,
    /// 最后活动时间
    pub last_activity: Instant,
    /// 重组后的完整 Payload
    pub reassembled_payload: Vec<u8>,
}

impl TcpStream {
    /// 创建新的 TCP 流
    pub fn new(five_tuple: FiveTuple, initial_seq: u32) -> Self {
        Self {
            five_tuple,
            segments: BTreeMap::new(),
            next_seq: initial_seq,
            status: ReassemblyStatus::InProgress,
            missing_count: 0,
            last_activity: Instant::now(),
            reassembled_payload: Vec::new(),
        }
    }

    /// 添加数据段
    pub fn add_segment(&mut self, seq: u32, data: Vec<u8>) {
        self.segments.insert(seq, data);
        self.last_activity = Instant::now();
    }

    /// 尝试重组数据
    pub fn reassemble(&mut self) -> bool {
        let mut current_seq = self.next_seq;
        let mut reassembled = Vec::new();

        // 按序列号顺序重组
        while let Some(data) = self.segments.remove(&current_seq) {
            reassembled.extend_from_slice(&data);
            current_seq = current_seq.wrapping_add(data.len() as u32);
        }

        if !reassembled.is_empty() {
            self.reassembled_payload.extend_from_slice(&reassembled);
            self.next_seq = current_seq;
            return true;
        }

        false
    }

    /// 检查是否超时
    pub fn is_timeout(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// 完成重组
    pub fn complete(&mut self) {
        if self.segments.is_empty() {
            self.status = ReassemblyStatus::Completed;
        } else {
            self.status = ReassemblyStatus::Failed(self.segments.len() as u32);
        }
    }
}

/// TCP 流重组器
pub struct TcpReassembler {
    /// 活跃的 TCP 流
    pub streams: BTreeMap<[u8; 40], TcpStream>, // 使用简化的 key（实际应该用 FiveTuple 的哈希）
    /// 超时时间
    pub timeout: Duration,
    /// 最大缓冲区大小（字节）
    pub max_buffer_size: usize,
}

impl TcpReassembler {
    /// 创建新的重组器
    pub fn new() -> Self {
        Self {
            streams: BTreeMap::new(),
            timeout: Duration::from_secs(60),
            max_buffer_size: 10 * 1024 * 1024, // 10MB
        }
    }

    /// 处理 TCP 包
    pub fn process_packet(
        &mut self,
        five_tuple: &FiveTuple,
        seq: u32,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        // 生成流的 key（简化实现，实际应该用 FiveTuple 的哈希）
        let key = [0u8; 40]; // 占位符

        // 获取或创建流
        let stream = self.streams.entry(key).or_insert_with(|| {
            TcpStream::new(five_tuple.clone(), seq)
        });

        // 添加数据段
        stream.add_segment(seq, data);

        // 尝试重组
        if stream.reassemble() {
            let payload = stream.reassembled_payload.clone();
            stream.reassembled_payload.clear();
            
            // 检查缓冲区大小
            if payload.len() > self.max_buffer_size {
                stream.status = ReassemblyStatus::Failed(1);
                return None;
            }
            
            return Some(payload);
        }

        None
    }

    /// 清理超时的流
    pub fn cleanup_timeout(&mut self) {
        let timeout = self.timeout;
        self.streams.retain(|_, stream| {
            if stream.is_timeout(timeout) {
                stream.complete();
                false
            } else {
                true
            }
        });
    }

    /// 获取流统计
    pub fn stats(&self) -> (usize, usize) {
        let active = self.streams.len();
        let completed = self.streams.values()
            .filter(|s| s.status == ReassemblyStatus::Completed)
            .count();
        (active, completed)
    }
}

impl Default for TcpReassembler {
    fn default() -> Self {
        Self::new()
    }
}

