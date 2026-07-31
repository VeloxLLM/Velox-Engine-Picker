//! 内存信息检测

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总物理内存 (MB)
    pub total: u64,
    /// 当前可用内存 (MB)
    pub available: u64,
}

/// 采集内存信息
#[must_use]
pub fn collect_memory_info(sys: &System) -> MemoryInfo {
    let total_bytes = sys.total_memory();
    let available_bytes = sys.available_memory();

    MemoryInfo {
        total: bytes_to_mb(total_bytes),
        available: bytes_to_mb(available_bytes),
    }
}

fn bytes_to_mb(bytes: u64) -> u64 {
    bytes / 1024 / 1024
}
