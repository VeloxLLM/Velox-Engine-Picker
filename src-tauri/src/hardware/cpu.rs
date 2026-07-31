//! CPU 信息检测

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// CPU 指令集架构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum CpuArch {
    X86_64,
    Aarch64,
    Arm,
    X86,
    Other,
}

impl From<CpuArch> for String {
    fn from(v: CpuArch) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for CpuArch {
    fn from(s: String) -> Self {
        match s.as_str() {
            "X86_64" => Self::X86_64,
            "Aarch64" => Self::Aarch64,
            "Arm" => Self::Arm,
            "X86" => Self::X86,
            _ => Self::Other,
        }
    }
}

impl CpuArch {
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64 (AMD64)",
            Self::Aarch64 => "AArch64 (ARM64)",
            Self::Arm => "ARM (32-bit)",
            Self::X86 => "x86 (32-bit)",
            Self::Other => "Unknown",
        }
    }

    #[must_use]
    pub const fn is_arm(&self) -> bool {
        matches!(self, Self::Aarch64 | Self::Arm)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub core_count: usize,
    pub frequency: u64,
    pub brand: CpuBrand,
    pub arch: CpuArch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum CpuBrand {
    Intel,
    Amd,
    Apple,
    Qualcomm,
    Other,
}

impl From<CpuBrand> for String {
    fn from(v: CpuBrand) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for CpuBrand {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Intel" => Self::Intel,
            "Amd" => Self::Amd,
            "Apple" => Self::Apple,
            "Qualcomm" => Self::Qualcomm,
            _ => Self::Other,
        }
    }
}

/// 采集 CPU 信息
#[must_use]
pub fn collect_cpu_info(sys: &System) -> CpuInfo {
    let cpus = sys.cpus();
    let core_count = cpus.len().max(1);

    let first_cpu = cpus.first();
    let name = first_cpu
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            first_cpu
                .map(|c| c.name().to_string())
                .unwrap_or_else(|| "Unknown CPU".to_string())
        });

    let vendor = first_cpu
        .map(|c| c.vendor_id().to_string())
        .unwrap_or_default();

    let frequency = {
        let sum: u64 = cpus.iter().map(|c| c.frequency()).sum();
        sum / core_count as u64
    };

    let brand = detect_brand(&name, &vendor);
    let arch = detect_arch();

    CpuInfo {
        name,
        vendor,
        core_count,
        frequency,
        brand,
        arch,
    }
}

fn detect_brand(name: &str, vendor: &str) -> CpuBrand {
    let n = name.to_ascii_lowercase();
    let v = vendor.to_ascii_lowercase();
    if n.contains("intel") || v.contains("intel") || v.contains("genuineintel") {
        CpuBrand::Intel
    } else if n.contains("amd") || v.contains("amd") || v.contains("authenticamd") {
        CpuBrand::Amd
    } else if n.contains("apple")
        || n.contains("m1 ")
        || n.contains("m2 ")
        || n.contains("m3 ")
        || n.contains("m4 ")
        || n.contains("m1,")
        || n.contains("m2,")
        || n.contains("m3,")
        || n.contains("m4,")
    {
        CpuBrand::Apple
    } else if n.contains("qualcomm")
        || n.contains("snapdragon")
        || v.contains("qualcomm")
        || name.contains("SC8")
        || name.contains("X Elite")
        || name.contains("X Plus")
    {
        CpuBrand::Qualcomm
    } else {
        CpuBrand::Other
    }
}

#[must_use]
fn detect_arch() -> CpuArch {
    #[cfg(target_arch = "x86_64")]
    {
        CpuArch::X86_64
    }
    #[cfg(target_arch = "aarch64")]
    {
        CpuArch::Aarch64
    }
    #[cfg(target_arch = "arm")]
    {
        CpuArch::Arm
    }
    #[cfg(target_arch = "x86")]
    {
        CpuArch::X86
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86"
    )))]
    {
        CpuArch::Other
    }
}