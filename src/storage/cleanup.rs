//! 数据清理（30 天保留）
//!
//! 定期清理过期的元数据和原始包文件

use crate::common::error::Result;
use crate::storage::metadata::MetadataStore;
use crate::storage::packet_store::PacketStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn};

/// 数据清理器
pub struct DataCleanup {
    /// 元数据存储
    metadata_store: Arc<MetadataStore>,
    /// 包存储
    packet_store: Arc<PacketStore>,
    /// 清理间隔
    cleanup_interval: Duration,
}

impl DataCleanup {
    /// 创建新的清理器
    pub fn new(
        metadata_store: Arc<MetadataStore>,
        packet_store: Arc<PacketStore>,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            metadata_store,
            packet_store,
            cleanup_interval,
        }
    }

    /// 启动清理任务
    pub async fn start(&self) {
        let mut interval = interval(self.cleanup_interval);

        loop {
            interval.tick().await;

            info!("开始执行数据清理任务...");

            // 清理过期的原始包文件
            match self.packet_store.cleanup_old_files() {
                Ok(count) => {
                    if count > 0 {
                        info!("已清理 {} 个过期的原始包文件", count);
                    }
                }
                Err(e) => {
                    warn!("清理原始包文件失败: {}", e);
                }
            }

            // 清理过期的元数据
            match self.cleanup_old_metadata() {
                Ok(count) => {
                    if count > 0 {
                        info!("已清理 {} 条过期的元数据", count);
                    }
                }
                Err(e) => {
                    warn!("清理元数据失败: {}", e);
                }
            }
        }
    }

    /// 清理过期的元数据
    fn cleanup_old_metadata(&self) -> Result<usize> {
        let conn = self.metadata_store.get_connection();
        let conn = conn.lock().unwrap();

        // 计算 30 天前的时间戳
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (30 * 24 * 60 * 60);

        // 删除过期数据
        let count = conn.execute(
            "DELETE FROM packet_metadata WHERE timestamp < ?1",
            rusqlite::params![cutoff_time * 1000], // 转换为毫秒
        )?;

        Ok(count)
    }
}
