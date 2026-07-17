//! DPI 引擎 Trait 定义
//!
//! 定义深度包检测引擎的统一接口

use crate::common::types::FiveTuple;
use std::collections::HashMap;

/// DPI 检测结果
#[derive(Debug, Clone)]
pub struct DpiResult {
    /// 识别的应用层协议
    pub app_protocol: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// DPI 引擎 Trait
pub trait DpiEngine {
    /// 检测数据包的应用层协议
    fn detect(&mut self, packet: &[u8], flow: &FiveTuple) -> DpiResult;
    
    /// 重置检测状态（用于新的流）
    fn reset(&mut self);
}

