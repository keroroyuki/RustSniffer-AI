//! 抓包引擎
//!
//! 实现网络数据包捕获的核心逻辑

use anyhow::{Context, Result};
use pcap::Capture;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::common::types::PacketInfo;

/// 抓包统计信息
#[derive(Debug, Clone)]
pub struct CaptureStats {
    /// 总包数
    pub total_packets: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 丢弃包数
    pub dropped_packets: u64,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
}

impl Default for CaptureStats {
    fn default() -> Self {
        Self {
            total_packets: 0,
            total_bytes: 0,
            dropped_packets: 0,
            start_time: Instant::now(),
            end_time: None,
        }
    }
}

impl CaptureStats {
    /// 计算抓包速率（包/秒）
    pub fn packets_per_second(&self) -> f64 {
        let duration = self
            .end_time
            .unwrap_or_else(Instant::now)
            .duration_since(self.start_time);
        if duration.as_secs() == 0 {
            0.0
        } else {
            self.total_packets as f64 / duration.as_secs_f64()
        }
    }

    /// 计算丢包率
    pub fn drop_rate(&self) -> f64 {
        if self.total_packets == 0 {
            0.0
        } else {
            self.dropped_packets as f64 / self.total_packets as f64 * 100.0
        }
    }
}

/// 抓包配置
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// 网卡名称
    pub interface: String,
    /// BPF 过滤器
    pub filter: Option<String>,
    /// 是否启用混杂模式
    pub promiscuous: bool,
    /// 最大包数（0 表示无限制）
    pub count: u64,
    /// 最大时长（秒，0 表示无限制）
    pub duration: u64,
    /// 缓冲区大小
    pub buffer_size: usize,
}

/// 抓包引擎
pub struct Sniffer {
    config: CaptureConfig,
    stats: CaptureStats,
    /// 采样相关字段
    sampling_enabled: bool,
    sampling_interval: u64,
    packet_counter: u64,
    last_sample_time: Instant,
}

impl Sniffer {
    /// 创建新的抓包引擎
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            stats: CaptureStats::default(),
            sampling_enabled: false,
            sampling_interval: 1,
            packet_counter: 0,
            last_sample_time: Instant::now(),
        }
    }

    /// 启动抓包
    pub async fn start(&mut self, tx: mpsc::Sender<PacketInfo>) -> Result<()> {
        info!("开始抓包，网卡: {}", self.config.interface);

        // 打开网卡
        let mut cap = Capture::from_device(self.config.interface.as_str())
            .context("无法打开网卡")?
            .promisc(self.config.promiscuous)
            .buffer_size(self.config.buffer_size as i32)
            .open()
            .context("无法激活网卡")?;

        // 应用 BPF 过滤器
        if let Some(ref filter) = self.config.filter {
            cap.filter(filter, true)
                .context(format!("应用 BPF 过滤器失败: {}", filter))?;
            info!("已应用 BPF 过滤器: {}", filter);
        }

        self.stats.start_time = Instant::now();
        let start_time = Instant::now();

        // 创建信号处理标志
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        // 启动 Ctrl+C 信号处理任务
        tokio::spawn(async move {
            if let Ok(()) = signal::ctrl_c().await {
                info!("收到 Ctrl+C 信号，正在停止抓包...");
                r.store(false, Ordering::SeqCst);
            }
        });

        // SIGHUP 信号处理（Unix only）
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal as unix_signal, SignalKind};
            let r_sighup = running.clone();
            tokio::spawn(async move {
                let mut sighup = unix_signal(SignalKind::hangup())
                    .expect("无法注册 SIGHUP 信号处理器");
                sighup.recv().await;
                info!("收到 SIGHUP 信号，重新加载配置...");
                match crate::config::loader::reload() {
                    Ok(new_config) => {
                        info!("配置已重新加载");
                        // 注意：实际应用中需要更新运行中的配置
                        // 这里仅作为示例
                    }
                    Err(e) => {
                        warn!("配置重新加载失败: {}", e);
                    }
                }
            });
        }

        // 抓包循环
        loop {
            // 检查是否收到停止信号
            if !running.load(Ordering::SeqCst) {
                info!("收到停止信号，正在退出...");
                break;
            }

            // 检查是否达到最大时长
            if self.config.duration > 0
                && start_time.elapsed() >= Duration::from_secs(self.config.duration)
            {
                info!("达到最大抓包时长: {} 秒", self.config.duration);
                break;
            }

            // 检查是否达到最大包数
            if self.config.count > 0 && self.stats.total_packets >= self.config.count {
                info!("达到最大抓包数: {} 包", self.config.count);
                break;
            }

            // 捕获数据包
            match cap.next_packet() {
                Ok(packet) => {
                    self.stats.total_packets += 1;
                    self.stats.total_bytes += packet.data.len() as u64;
                    self.packet_counter += 1;

                    // 动态调整采样率（每秒检查一次）
                    if self.last_sample_time.elapsed() >= Duration::from_secs(1) {
                        let current_rate = self.stats.packets_per_second();
                        
                        // 如果流量超过 100,000 包/秒，启用采样
                        if current_rate > 100_000.0 && !self.sampling_enabled {
                            self.sampling_enabled = true;
                            self.sampling_interval = (current_rate / 100_000.0).ceil() as u64;
                            warn!(
                                "流量过高 ({:.0} 包/秒)，已启用采样模式，实际流量为显示值的 {} 倍",
                                current_rate, self.sampling_interval
                            );
                        }
                        
                        // 动态调整采样间隔
                        if self.sampling_enabled {
                            let new_interval = (current_rate / 100_000.0).ceil() as u64;
                            if new_interval != self.sampling_interval {
                                self.sampling_interval = new_interval;
                                info!("采样率调整为 1/{}", self.sampling_interval);
                            }
                        }
                        
                        self.last_sample_time = Instant::now();
                    }

                    // 采样逻辑：如果启用采样，只保留每 N 个包中的 1 个
                    if self.sampling_enabled && !self.packet_counter.is_multiple_of(self.sampling_interval) {
                        continue;
                    }

                    let packet_info = PacketInfo {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        raw_bytes: packet.data.to_vec(),
                        interface_id: self.config.interface.clone(),
                    };

                    // 发送到通道
                    if tx.send(packet_info).await.is_err() {
                        warn!("数据包发送通道已关闭");
                        break;
                    }
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // 超时继续
                    continue;
                }
                Err(e) => {
                    warn!("抓包错误: {}", e);
                    self.stats.dropped_packets += 1;
                }
            }
        }

        self.stats.end_time = Some(Instant::now());
        info!(
            "抓包结束，总包数: {}, 总字节: {}, 丢包: {}, 速率: {:.2} 包/秒",
            self.stats.total_packets,
            self.stats.total_bytes,
            self.stats.dropped_packets,
            self.stats.packets_per_second()
        );

        Ok(())
    }

    /// 获取统计信息
    pub fn stats(&self) -> &CaptureStats {
        &self.stats
    }
}

