//! 采集层模块
//!
//! 负责网络数据包捕获，包括网卡枚举、BPF 过滤器、抓包引擎

pub mod bpf;
pub mod interface;
pub mod sniffer;

// 导出公共 API
pub use interface::{list_interfaces, select_interface, InterfaceInfo};
pub use sniffer::{CaptureConfig, CaptureStats, Sniffer};
