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

pub struct GpuDetectionResult {
    pub gpus: Vec<GpuInfo>,
    pub warning: Option<String>,
}

#[must_use]
pub fn collect_gpu_info() -> GpuDetectionResult {
    #[cfg(target_os = "windows")]
    {
        match collect_dxgi_gpu_info() {
            Ok(gpus) if !gpus.is_empty() => GpuDetectionResult {
                gpus,
                warning: None,
            },
            Ok(_) => GpuDetectionResult {
                gpus: Vec::new(),
                warning: None,
            },
            Err(error) => GpuDetectionResult {
                gpus: Vec::new(),
                warning: Some(format!("DXGI GPU 检测失败: {error}")),
            },
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        match pollster::block_on(try_collect_gpu_info()) {
            Some(gpus) if !gpus.is_empty() => GpuDetectionResult {
                gpus,
                warning: None,
            },
            Some(_) => GpuDetectionResult {
                gpus: Vec::new(),
                warning: None,
            },
            None => GpuDetectionResult {
                gpus: Vec::new(),
                warning: Some("wgpu GPU 检测失败".to_string()),
            },
        }
    }
}

#[cfg(target_os = "windows")]
fn collect_dxgi_gpu_info() -> Result<Vec<GpuInfo>, String> {
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    };

    let wgpu_adapters = pollster::block_on(try_collect_gpu_info()).unwrap_or_default();
    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(|error| error.to_string())?;
    let mut results = Vec::new();
    let mut index = 0;

    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => break,
        };
        let desc = unsafe { adapter.GetDesc1() }.map_err(|error| error.to_string())?;
        index += 1;

        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }

        let name = String::from_utf16_lossy(
            &desc.Description[..desc
                .Description
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(desc.Description.len())],
        );
        let vendor = detect_vendor(desc.VendorId, &name);
        let matched = wgpu_adapters.iter().find(|gpu| {
            gpu.vendor == vendor
                && (gpu.name.eq_ignore_ascii_case(&name)
                    || gpu
                        .name
                        .to_ascii_lowercase()
                        .contains(&name.to_ascii_lowercase())
                    || name
                        .to_ascii_lowercase()
                        .contains(&gpu.name.to_ascii_lowercase()))
        });
        let gpu_type = matched.map_or_else(
            || {
                if desc.DedicatedVideoMemory > 512 * 1024 * 1024 {
                    GpuType::Discrete
                } else {
                    GpuType::Integrated
                }
            },
            |gpu| gpu.gpu_type.clone(),
        );
        let backend = matched.map_or(GpuBackend::Dx12, |gpu| gpu.backend.clone());
        let dedicated_mb = desc.DedicatedVideoMemory as u64 / 1024 / 1024;

        results.push(GpuInfo {
            name,
            vendor,
            gpu_type,
            vram: (dedicated_mb > 0).then_some(dedicated_mb),
            backend,
            device_id: (desc.VendorId, desc.DeviceId),
        });
    }

    Ok(results)
}

async fn try_collect_gpu_info() -> Option<Vec<GpuInfo>> {
    #[cfg(feature = "v3")]
    let selected_backends = wgpu::Backends::PRIMARY;

    #[cfg(all(feature = "v2", not(feature = "v3")))]
    let selected_backends = wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::METAL;

    #[cfg(all(feature = "v1", not(feature = "v2")))]
    let selected_backends = wgpu::Backends::VULKAN | wgpu::Backends::DX12;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: selected_backends,
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(selected_backends);

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
        // wgpu does not expose physical VRAM. Platform-native collectors fill
        // this field where possible; never guess capacity from a product name.
        let vram = None;

        let is_dup = results
            .iter()
            .any(|e: &GpuInfo| e.vendor == vendor && e.name == info.name && e.gpu_type == gpu_type);
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
        wgpu::Backend::Gl => GpuBackend::Gl,
        wgpu::Backend::BrowserWebGpu => GpuBackend::BrowserWebGpu,
        _ => GpuBackend::Other,
    }
}
