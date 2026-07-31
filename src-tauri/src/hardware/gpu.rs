//! GPU / iGPU 信息检测（基于 wgpu 适配器枚举）

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Microsoft,
    Other,
}

impl From<GpuVendor> for String {
    fn from(v: GpuVendor) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for GpuVendor {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Nvidia" => Self::Nvidia,
            "Amd" => Self::Amd,
            "Intel" => Self::Intel,
            "Apple" => Self::Apple,
            "Qualcomm" => Self::Qualcomm,
            "Microsoft" => Self::Microsoft,
            _ => Self::Other,
        }
    }
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nvidia => f.write_str("NVIDIA"),
            Self::Amd => f.write_str("AMD"),
            Self::Intel => f.write_str("Intel"),
            Self::Apple => f.write_str("Apple"),
            Self::Qualcomm => f.write_str("Qualcomm"),
            Self::Microsoft => f.write_str("Microsoft"),
            Self::Other => f.write_str("Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum GpuType {
    Discrete,
    Integrated,
    Other,
}

impl From<GpuType> for String {
    fn from(v: GpuType) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for GpuType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Discrete" => Self::Discrete,
            "Integrated" => Self::Integrated,
            _ => Self::Other,
        }
    }
}

impl fmt::Display for GpuType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discrete => f.write_str("Discrete GPU"),
            Self::Integrated => f.write_str("Integrated GPU"),
            Self::Other => f.write_str("Virtual/Other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum GpuBackend {
    Vulkan,
    Metal,
    Dx12,
    Dx11,
    Gl,
    BrowserWebGpu,
    Other,
}

impl From<GpuBackend> for String {
    fn from(v: GpuBackend) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for GpuBackend {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Vulkan" => Self::Vulkan,
            "Metal" => Self::Metal,
            "Dx12" => Self::Dx12,
            "Dx11" => Self::Dx11,
            "Gl" => Self::Gl,
            "BrowserWebGpu" => Self::BrowserWebGpu,
            _ => Self::Other,
        }
    }
}

impl fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vulkan => f.write_str("Vulkan"),
            Self::Metal => f.write_str("Metal"),
            Self::Dx12 => f.write_str("DirectX 12"),
            Self::Dx11 => f.write_str("DirectX 11"),
            Self::Gl => f.write_str("OpenGL"),
            Self::BrowserWebGpu => f.write_str("WebGPU"),
            Self::Other => f.write_str("Other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub gpu_type: GpuType,
    pub vram: Option<u64>,
    pub backend: GpuBackend,
    pub device_id: (u32, u32),
}

#[must_use]
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    match pollster::block_on(try_collect_gpu_info()) {
        Some(gpus) if !gpus.is_empty() => gpus,
        _ => {
            log::warn!("GPU 信息检测失败");
            Vec::new()
        }
    }
}

async fn try_collect_gpu_info() -> Option<Vec<GpuInfo>> {
    #[cfg(feature = "v3")]
    let selected_backends = wgpu::Backends::PRIMARY;

    #[cfg(all(feature = "v2", not(feature = "v3")))]
    let selected_backends = wgpu::Backends::VULKAN
        | wgpu::Backends::DX12
        | wgpu::Backends::DX11
        | wgpu::Backends::METAL;

    #[cfg(all(feature = "v1", not(feature = "v2")))]
    let selected_backends = wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::DX11;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: selected_backends,
        ..Default::default()
    });

    let adapters: Vec<_> = instance.enumerate_adapters(selected_backends).collect();

    if adapters.is_empty() {
        log::warn!("wgpu 未枚举到任何图形适配器");
        return None;
    }

    let mut results = Vec::with_capacity(adapters.len());
    for adapter in &adapters {
        let info = adapter.get_info();

        let vendor = detect_vendor(info.vendor, &info.name);
        let gpu_type = match info.device_type {
            wgpu::DeviceType::DiscreteGpu => GpuType::Discrete,
            wgpu::DeviceType::IntegratedGpu => GpuType::Integrated,
            _ => GpuType::Other,
        };
        let backend = map_backend(info.backend);
        let vram = estimate_vram_from_name(&info.name);

        let is_dup = results.iter().any(|e: &GpuInfo| {
            e.vendor == vendor && e.name == info.name && e.gpu_type == gpu_type
        });
        if is_dup {
            continue;
        }

        results.push(GpuInfo {
            name: info.name.clone(),
            vendor,
            gpu_type,
            vram,
            backend,
            device_id: (info.vendor, info.device),
        });
    }

    Some(results)
}

fn detect_vendor(vendor_id: u32, name: &str) -> GpuVendor {
    match vendor_id {
        0x10DE => GpuVendor::Nvidia,
        0x1002 => GpuVendor::Amd,
        0x8086 => GpuVendor::Intel,
        0x106B => GpuVendor::Apple,
        0x5143 => GpuVendor::Qualcomm,
        0x1414 => GpuVendor::Microsoft,
        _ => {
            let n = name.to_ascii_lowercase();
            if n.contains("nvidia") {
                GpuVendor::Nvidia
            } else if n.contains("amd") || n.contains("radeon") {
                GpuVendor::Amd
            } else if n.contains("intel") {
                GpuVendor::Intel
            } else if n.contains("apple") {
                GpuVendor::Apple
            } else if n.contains("qualcomm") || n.contains("adreno") {
                GpuVendor::Qualcomm
            } else {
                GpuVendor::Other
            }
        }
    }
}

fn map_backend(b: wgpu::Backend) -> GpuBackend {
    match b {
        wgpu::Backend::Vulkan => GpuBackend::Vulkan,
        wgpu::Backend::Metal => GpuBackend::Metal,
        wgpu::Backend::Dx12 => GpuBackend::Dx12,
        wgpu::Backend::Dx11 => GpuBackend::Dx11,
        wgpu::Backend::Gl => GpuBackend::Gl,
        wgpu::Backend::BrowserWebGpu => GpuBackend::BrowserWebGpu,
        _ => GpuBackend::Other,
    }
}

fn estimate_vram_from_name(name: &str) -> Option<u64> {
    let n = name.to_ascii_lowercase();
    for marker in ["gb", " gb"] {
        let mut search_from = 0;
        while let Some(idx) = n[search_from..].find(marker) {
            let abs_idx = search_from + idx;
            let start = if abs_idx >= 6 { abs_idx - 6 } else { 0 };
            let prefix = &n[start..abs_idx];
            let digits: String = prefix
                .chars()
                .rev()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if let Ok(gb) = digits.parse::<u64>() {
                if gb >= 1 && gb <= 192 {
                    return Some(gb * 1024);
                }
            }
            search_from = abs_idx + marker.len();
        }
    }
    None
}