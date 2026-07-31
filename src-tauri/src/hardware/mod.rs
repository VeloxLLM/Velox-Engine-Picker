//! 硬件检测模块
//!
//! 负责采集本机 CPU、内存、GPU/iGPU 等硬件信息。

pub mod cpu;
pub mod gpu;
pub mod memory;

pub use cpu::{collect_cpu_info, CpuArch, CpuBrand, CpuInfo};
pub use gpu::{collect_gpu_info, GpuBackend, GpuInfo, GpuType, GpuVendor};
pub use memory::{collect_memory_info, MemoryInfo};

use serde::Serialize;
use std::fmt;

/// 汇总的硬件信息
#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
}

impl HardwareInfo {
    /// 采集所有硬件信息
    pub fn collect() -> Self {
        Self {
            cpu: collect_cpu_info(),
            memory: collect_memory_info(),
            gpus: collect_gpu_info(),
        }
    }

    /// 是否检测到独立显卡
    pub fn has_discrete_gpu(&self) -> bool {
        self.gpus.iter().any(|g| g.gpu_type == GpuType::Discrete)
    }

    /// 是否检测到 Intel 显卡（集显或 Arc 独显）
    pub fn has_intel_gpu(&self) -> bool {
        self.gpus.iter().any(|g| g.vendor == GpuVendor::Intel)
    }

    /// 是否检测到 NVIDIA 独立显卡
    pub fn has_nvidia_discrete(&self) -> bool {
        self.gpus
            .iter()
            .any(|g| g.vendor == GpuVendor::Nvidia && g.gpu_type == GpuType::Discrete)
    }

    /// 是否检测到 AMD 独立显卡
    pub fn has_amd_discrete(&self) -> bool {
        self.gpus
            .iter()
            .any(|g| g.vendor == GpuVendor::Amd && g.gpu_type == GpuType::Discrete)
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