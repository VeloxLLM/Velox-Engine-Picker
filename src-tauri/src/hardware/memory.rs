//! 内存信息检测

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    /// 总物理内存 (MB)
    pub total: u64,
    /// 当前可用内存 (MB)
    pub available: u64,
}

/// 采集内存信息
#[must_use]
pub fn collect_memory_info() -> MemoryInfo {
    let mut sys = System::new();
    sys.refresh_memory();

    let total_kb = sys.total_memory();
    let available_kb = sys.available_memory();

    MemoryInfo {
        total: kb_to_mb(total_kb),
        available: kb_to_mb(available_kb),
    }
}

fn kb_to_mb(kb: u64) -> u64 {
    kb / 1024
}