//! 硬件检测模块
//!
//! 负责采集本机 CPU、内存、GPU/iGPU 等硬件信息。

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod platform;
pub mod runtime;

#[cfg(test)]
pub use cpu::CpuBrand;
pub use cpu::{collect_cpu_info, CpuArch, CpuInfo};
pub use gpu::{collect_gpu_info, GpuBackend, GpuInfo, GpuType, GpuVendor};
pub use memory::{collect_memory_info, MemoryInfo};
pub use platform::{collect_platform_info, PlatformInfo};
pub use runtime::collect_runtime_availability;

use crate::engine::types::EngineAvailability;

use serde::{Deserialize, Serialize};
use std::fmt;

/// 汇总的硬件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub platform: PlatformInfo,
    pub availability: Vec<EngineAvailability>,
    pub detection_warnings: Vec<String>,
}

impl HardwareInfo {
    /// 采集所有硬件信息
    pub fn collect() -> Self {
        log::info!("开始硬件检测...");
        let mut sys = sysinfo::System::new();
        sys.refresh_all();
        let cpu = collect_cpu_info(&sys);
        let gpu_result = collect_gpu_info();
        let gpus = gpu_result.gpus;
        let memory = collect_memory_info(&sys);
        let platform = collect_platform_info();
        let availability = collect_runtime_availability(&gpus);
        let hw = Self {
            cpu,
            gpus,
            memory,
            platform,
            availability,
            detection_warnings: gpu_result.warning.into_iter().collect(),
        };
        log::info!(
            "硬件检测完成: CPU={}, {} GPU(s), {} MB RAM",
            hw.cpu.name,
            hw.gpus.len(),
            hw.memory.total
        );
        hw
    }
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self::collect()
    }
}

impl fmt::Display for HardwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPU: {}", self.cpu.name)?;
        writeln!(
            f,
            "Memory: {} MB (available: {} MB)",
            self.memory.total, self.memory.available
        )?;
        for (i, g) in self.gpus.iter().enumerate() {
            writeln!(
                f,
                "GPU[{}]: {} ({:?}, {:?}, vram: {} MB)",
                i,
                g.name,
                g.vendor,
                g.gpu_type,
                g.vram.unwrap_or(0)
            )?;
        }
        Ok(())
    }
}
