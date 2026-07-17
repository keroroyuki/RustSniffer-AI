//! nDPI DPI 引擎存根实现
//!
//! 注意：完整的 nDPI 集成需要安装 nDPI C 库并配置 FFI 绑定
//! 当前提供存根实现，返回 "Unknown" 协议

use crate::common::types::FiveTuple;
use crate::dpi::engine::{DpiEngine, DpiResult};
use std::collections::HashMap;

/// nDPI DPI 引擎存根实现
pub struct NdpiDpiEngine {
    _placeholder: (),
}

impl NdpiDpiEngine {
    /// 创建新的 nDPI 引擎（存根实现）
    pub fn new() -> Result<Self, String> {
        Ok(Self { _placeholder: () })
    }
}

impl DpiEngine for NdpiDpiEngine {
    fn detect(&mut self, _packet: &[u8], _flow: &FiveTuple) -> DpiResult {
        // 存根实现：返回 Unknown
        // 完整的 nDPI 集成需要：
        // 1. 安装 nDPI 库（https://github.com/ntop/nDPI）
        // 2. 使用 bindgen 生成 FFI 绑定
        // 3. 在 build.rs 中链接 nDPI 库
        DpiResult {
            app_protocol: "Unknown".to_string(),
            confidence: 0.0,
            metadata: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        // 存根实现：无需清理
    }
}
