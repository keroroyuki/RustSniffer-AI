//! 原始包存储（PCAPNG）
//!
//! 使用 PCAPNG 格式存储原始数据包，支持 30 天自动清理

use crate::common::error::Result;
use crate::protocol::parser::ParsedPacket;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
const PCAPNG_MAGIC: u32 = 0x0A0D0D0A;
/// Section Header Block 类型
const SHB_TYPE: u32 = 0x0A0D0D0A;
/// Interface Description Block 类型
const IDB_TYPE: u32 = 0x00000001;
/// Enhanced Packet Block 类型
const EPB_TYPE: u32 = 0x00000006;

/// 原始包存储管理器
pub struct PacketStore {
    /// 存储目录
    storage_dir: PathBuf,
    /// 当前写入器
    writer: Arc<Mutex<Option<BufWriter<File>>>>,
    /// 当前文件路径
    current_file: Arc<Mutex<Option<PathBuf>>>,
    /// 保留天数
    retention_days: u32,
}

impl PacketStore {
    /// 创建新的包存储
    pub fn new<P: AsRef<Path>>(storage_dir: P, retention_days: u32) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();

        // 创建存储目录
        fs::create_dir_all(&storage_dir)?;

        Ok(Self {
            storage_dir,
            writer: Arc::new(Mutex::new(None)),
            current_file: Arc::new(Mutex::new(None)),
            retention_days,
        })
    }

    /// 初始化新的 PCAPNG 文件
    pub fn init_file(&self) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let filename = format!("packets_{}.pcapng", timestamp);
        let file_path = self.storage_dir.join(filename);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;

        let mut writer = BufWriter::new(file);

        // 写入 Section Header Block (SHB)
        self.write_shb(&mut writer)?;

        // 写入 Interface Description Block (IDB)
        self.write_idb(&mut writer)?;

        writer.flush()?;

        // 更新当前写入器
        let mut current_writer = self.writer.lock().unwrap();
        *current_writer = Some(writer);

        let mut current_file = self.current_file.lock().unwrap();
        *current_file = Some(file_path.clone());

        Ok(file_path)
    }

    /// 写入 Section Header Block
    fn write_shb(&self, writer: &mut BufWriter<File>) -> Result<()> {
        // Block Type
        writer.write_all(&SHB_TYPE.to_le_bytes())?;
        // Block Total Length (placeholder, will be updated)
        let block_length_pos = writer.get_ref().metadata().unwrap().len();
        writer.write_all(&0u32.to_le_bytes())?;
        // Byte-Order Magic
        writer.write_all(&0x1A2B3C4Du32.to_le_bytes())?;
        // Major Version
        writer.write_all(&1u16.to_le_bytes())?;
        // Minor Version
        writer.write_all(&0u16.to_le_bytes())?;
        // Section Length (unspecified)
        writer.write_all(&(-1i64).to_le_bytes())?;

        // Calculate and update block length
        let current_pos = writer.get_ref().metadata().unwrap().len();
        let block_length = (current_pos - block_length_pos + 4) as u32;

        // Write block length again at the end
        writer.write_all(&block_length.to_le_bytes())?;

        Ok(())
    }

    /// 写入 Interface Description Block
    fn write_idb(&self, writer: &mut BufWriter<File>) -> Result<()> {
        // Block Type
        writer.write_all(&IDB_TYPE.to_le_bytes())?;
        // Block Total Length
        writer.write_all(&20u32.to_le_bytes())?;
        // LinkType (Ethernet)
        writer.write_all(&1u16.to_le_bytes())?;
        // Reserved
        writer.write_all(&0u16.to_le_bytes())?;
        // SnapLen
        writer.write_all(&65535u32.to_le_bytes())?;
        // Block Total Length (repeated)
        writer.write_all(&20u32.to_le_bytes())?;

        Ok(())
    }

    /// 写入数据包
    pub fn write_packet(&self, packet: &ParsedPacket) -> Result<()> {
        let mut writer_guard = self.writer.lock().unwrap();

        if let Some(writer) = writer_guard.as_mut() {
            self.write_epb(writer, packet)?;
            writer.flush()?;
        } else {
            // 如果没有打开的文件，创建一个新的
            drop(writer_guard);
            self.init_file()?;
            let mut writer_guard = self.writer.lock().unwrap();
            if let Some(writer) = writer_guard.as_mut() {
                self.write_epb(writer, packet)?;
                writer.flush()?;
            }
        }

        Ok(())
    }

    /// 写入 Enhanced Packet Block
    fn write_epb(&self, writer: &mut BufWriter<File>, packet: &ParsedPacket) -> Result<()> {
        let packet_data = &packet.raw_bytes;
        let padded_len = (packet_data.len() + 3) & !3; // 4 字节对齐

        // Block Type
        writer.write_all(&EPB_TYPE.to_le_bytes())?;

        // Block Total Length (placeholder)
        let block_length_pos = writer.get_ref().metadata().unwrap().len();
        writer.write_all(&0u32.to_le_bytes())?;

        // Interface ID
        writer.write_all(&0u32.to_le_bytes())?;

        // Timestamp (High)
        let timestamp_us = (packet.timestamp * 1000) as u64;
        let timestamp_high = (timestamp_us >> 32) as u32;
        writer.write_all(&timestamp_high.to_le_bytes())?;

        // Timestamp (Low)
        let timestamp_low = (timestamp_us & 0xFFFFFFFF) as u32;
        writer.write_all(&timestamp_low.to_le_bytes())?;

        // Captured Packet Length
        writer.write_all(&(packet_data.len() as u32).to_le_bytes())?;

        // Original Packet Length
        writer.write_all(&(packet_data.len() as u32).to_le_bytes())?;

        // Packet Data
        writer.write_all(packet_data)?;

        // Padding
        let padding = vec![0u8; padded_len - packet_data.len()];
        writer.write_all(&padding)?;

        // Calculate and write block length
        let current_pos = writer.get_ref().metadata().unwrap().len();
        let block_length = (current_pos - block_length_pos + 4) as u32;
        writer.write_all(&block_length.to_le_bytes())?;

        Ok(())
    }

    /// 获取存储目录
    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    /// 清理过期文件
    pub fn cleanup_old_files(&self) -> Result<usize> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (self.retention_days as u64 * 24 * 60 * 60);

        let mut removed_count = 0;

        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("pcapng") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_time) = modified.duration_since(UNIX_EPOCH) {
                            if modified_time.as_secs() < cutoff_time {
                                fs::remove_file(&path)?;
                                removed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(removed_count)
    }
}
