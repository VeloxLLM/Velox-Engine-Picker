//! GPU / iGPU 信息检测（基于 wgpu 适配器枚举）

use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::collections::{HashMap, HashSet};
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
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_REMOTE, DXGI_ADAPTER_FLAG_SOFTWARE,
    };

    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(|error| error.to_string())?;
    let physical_adapter_counts = collect_present_pci_display_adapters().unwrap_or_else(|error| {
        log::warn!("Windows 物理显示设备枚举失败，将仅使用 DXGI 结果: {error}");
        HashMap::new()
    });
    let mut kept_adapter_counts = HashMap::new();
    let mut results = Vec::new();
    let mut seen_adapter_luids = HashSet::new();
    let mut index = 0;

    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => break,
        };
        let desc = unsafe { adapter.GetDesc1() }.map_err(|error| error.to_string())?;
        index += 1;

        let adapter_luid = (desc.AdapterLuid.HighPart, desc.AdapterLuid.LowPart);
        if !is_new_dxgi_adapter(&mut seen_adapter_luids, adapter_luid) {
            log::info!("忽略 DXGI 重复适配器: LUID={adapter_luid:?}");
            continue;
        }

        if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 | DXGI_ADAPTER_FLAG_REMOTE.0 as u32)
            != 0
        {
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
        let dedicated_mb = desc.DedicatedVideoMemory as u64 / 1024 / 1024;
        if !is_physical_adapter(&vendor, &name, dedicated_mb) {
            log::info!("忽略虚拟或显示专用适配器: {name}");
            continue;
        }
        if is_excess_dxgi_adapter(
            &physical_adapter_counts,
            &mut kept_adapter_counts,
            (desc.VendorId, desc.DeviceId),
        ) {
            log::info!(
                "忽略超过 Windows 物理设备数量的 DXGI 重复适配器: {name} ({:04x}:{:04x})",
                desc.VendorId,
                desc.DeviceId
            );
            continue;
        }
        let gpu_type = classify_gpu(&vendor, &name, dedicated_mb);

        results.push(GpuInfo {
            name,
            vendor,
            gpu_type,
            vram: (dedicated_mb > 0).then_some(dedicated_mb),
            backend: GpuBackend::Dx12,
            device_id: (desc.VendorId, desc.DeviceId),
        });
    }

    Ok(results)
}

#[cfg(target_os = "windows")]
fn collect_present_pci_display_adapters() -> Result<HashMap<(u32, u32), usize>, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, DIGCF_PRESENT, GUID_DEVCLASS_DISPLAY, SP_DEVINFO_DATA,
    };
    use windows::Win32::Foundation::HWND;

    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            Some(&GUID_DEVCLASS_DISPLAY),
            PCWSTR::null(),
            HWND::default(),
            DIGCF_PRESENT,
        )
    }
    .map_err(|error| error.to_string())?;

    let mut counts = HashMap::new();
    let mut index = 0;
    loop {
        let mut device_info = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        if unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info) }.is_err() {
            break;
        }
        index += 1;

        let mut instance_id = [0_u16; 512];
        if unsafe {
            SetupDiGetDeviceInstanceIdW(device_info_set, &device_info, Some(&mut instance_id), None)
        }
        .is_err()
        {
            continue;
        }
        let length = instance_id
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(instance_id.len());
        let instance_id = String::from_utf16_lossy(&instance_id[..length]);
        if let Some(signature) = parse_pci_adapter_signature(&instance_id) {
            *counts.entry(signature).or_insert(0) += 1;
        }
    }

    let _ = unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
    Ok(counts)
}

fn parse_pci_adapter_signature(instance_id: &str) -> Option<(u32, u32)> {
    let normalized = instance_id.to_ascii_uppercase();
    if !normalized.starts_with("PCI\\") {
        return None;
    }
    let vendor_id = normalized
        .split(['&', '\\'])
        .find_map(|part| part.strip_prefix("VEN_"))
        .and_then(|value| u32::from_str_radix(value, 16).ok())?;
    let device_id = normalized
        .split(['&', '\\'])
        .find_map(|part| part.strip_prefix("DEV_"))
        .and_then(|value| u32::from_str_radix(value, 16).ok())?;
    Some((vendor_id, device_id))
}

fn is_excess_dxgi_adapter(
    physical_adapter_counts: &HashMap<(u32, u32), usize>,
    kept_adapter_counts: &mut HashMap<(u32, u32), usize>,
    signature: (u32, u32),
) -> bool {
    let Some(physical_count) = physical_adapter_counts.get(&signature) else {
        return false;
    };
    let kept_count = kept_adapter_counts.entry(signature).or_insert(0);
    if *kept_count >= *physical_count {
        return true;
    }
    *kept_count += 1;
    false
}

#[cfg(target_os = "windows")]
fn is_new_dxgi_adapter(
    seen_adapter_luids: &mut HashSet<(i32, u32)>,
    adapter_luid: (i32, u32),
) -> bool {
    seen_adapter_luids.insert(adapter_luid)
}

fn is_physical_adapter(vendor: &GpuVendor, name: &str, dedicated_mb: u64) -> bool {
    let lower = name.to_ascii_lowercase();
    let virtual_name = [
        "virtual",
        "indirect",
        "remote",
        "oray",
        "spacedesk",
        "parsec",
        "iddsample",
    ]
    .iter()
    .any(|marker| lower.contains(marker));
    if virtual_name {
        return false;
    }
    !matches!(vendor, GpuVendor::Other | GpuVendor::Microsoft) || dedicated_mb > 0
}

fn classify_gpu(vendor: &GpuVendor, name: &str, dedicated_mb: u64) -> GpuType {
    match vendor {
        GpuVendor::Nvidia => GpuType::Discrete,
        GpuVendor::Intel if name.to_ascii_lowercase().contains("arc") || dedicated_mb > 512 => {
            GpuType::Discrete
        }
        GpuVendor::Amd if dedicated_mb > 512 => GpuType::Discrete,
        GpuVendor::Intel | GpuVendor::Amd | GpuVendor::Apple | GpuVendor::Qualcomm => {
            GpuType::Integrated
        }
        _ if dedicated_mb > 512 => GpuType::Discrete,
        _ => GpuType::Other,
    }
}

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_oray_indirect_display_adapter() {
        assert!(!is_physical_adapter(
            &GpuVendor::Other,
            "OrayIddDriver Device",
            0
        ));
    }

    #[test]
    fn keeps_nvidia_physical_adapter() {
        assert!(is_physical_adapter(
            &GpuVendor::Nvidia,
            "NVIDIA GeForce RTX 3060 Ti",
            8192
        ));
        assert_eq!(
            classify_gpu(&GpuVendor::Nvidia, "NVIDIA GeForce RTX 3060 Ti", 8192),
            GpuType::Discrete
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn duplicate_dxgi_luid_is_only_kept_once() {
        let mut seen = HashSet::new();
        assert!(is_new_dxgi_adapter(&mut seen, (42, 7)));
        assert!(!is_new_dxgi_adapter(&mut seen, (42, 7)));
        assert!(is_new_dxgi_adapter(&mut seen, (42, 8)));
    }

    #[test]
    fn parses_windows_pci_display_instance_id() {
        assert_eq!(
            parse_pci_adapter_signature(
                r"PCI\VEN_10DE&DEV_2489&SUBSYS_405A1458&REV_A1\4&739B9B0&0&0016"
            ),
            Some((0x10DE, 0x2489))
        );
        assert_eq!(parse_pci_adapter_signature(r"ROOT\DISPLAY\0000"), None);
    }

    #[test]
    fn caps_dxgi_duplicates_to_present_physical_device_count() {
        let physical = HashMap::from([((0x10DE, 0x2489), 1)]);
        let mut kept = HashMap::new();
        assert!(!is_excess_dxgi_adapter(
            &physical,
            &mut kept,
            (0x10DE, 0x2489)
        ));
        assert!(is_excess_dxgi_adapter(
            &physical,
            &mut kept,
            (0x10DE, 0x2489)
        ));
    }

    #[test]
    fn classifies_intel_integrated_and_arc_discrete() {
        assert_eq!(
            classify_gpu(&GpuVendor::Intel, "Intel UHD Graphics", 128),
            GpuType::Integrated
        );
        assert_eq!(
            classify_gpu(&GpuVendor::Intel, "Intel Arc A770", 16384),
            GpuType::Discrete
        );
    }
}
