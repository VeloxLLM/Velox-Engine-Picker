//! Conservative runtime probes. A backend is only `Ready` when both compatible
//! hardware and a concrete local runtime signal are present.

use std::path::{Path, PathBuf};

use crate::engine::types::{
    AvailabilityStatus, BackendType, EngineAvailability, EngineBackendPair, InferenceEngine,
};

use super::{GpuBackend, GpuInfo, GpuVendor};

fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

fn find_on_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

fn system_file(name: &str) -> Option<PathBuf> {
    let root = std::env::var_os("SystemRoot")?;
    let path = Path::new(&root).join("System32").join(name);
    path.is_file().then_some(path)
}

fn availability(
    engine: InferenceEngine,
    backend: BackendType,
    status: AvailabilityStatus,
    evidence: impl Into<String>,
) -> EngineAvailability {
    EngineAvailability {
        target: EngineBackendPair::new(engine, backend),
        status,
        version: None,
        evidence: evidence.into(),
    }
}

#[must_use]
pub fn collect_runtime_availability(gpus: &[GpuInfo]) -> Vec<EngineAvailability> {
    let is_windows = cfg!(target_os = "windows");
    let has_nvidia = gpus.iter().any(|g| g.vendor == GpuVendor::Nvidia);
    let has_amd = gpus.iter().any(|g| g.vendor == GpuVendor::Amd);
    let has_intel = gpus.iter().any(|g| g.vendor == GpuVendor::Intel);
    let has_apple = gpus.iter().any(|g| g.vendor == GpuVendor::Apple);
    let has_d3d12 = gpus.iter().any(|g| g.backend == GpuBackend::Dx12);
    let has_vulkan = gpus.iter().any(|g| g.backend == GpuBackend::Vulkan);

    let openvino = env_dir("OPENVINO_INSTALL_DIR")
        .or_else(|| find_on_path(&["openvino.dll", "libopenvino.so", "libopenvino.dylib"]));
    let llama = find_on_path(&["llama-cli.exe", "llama-cli"]);
    let cuda_driver = system_file("nvcuda.dll").or_else(|| find_on_path(&["libcuda.so"]));
    let tensorrt =
        env_dir("TENSORRT_ROOT").or_else(|| find_on_path(&["nvinfer.dll", "libnvinfer.so"]));
    let rocm = env_dir("HIP_PATH")
        .or_else(|| env_dir("ROCM_PATH"))
        .or_else(|| find_on_path(&["amdhip64.dll", "libamdhip64.so"]));
    let directml = system_file("DirectML.dll");
    let onnx = find_on_path(&[
        "onnxruntime.dll",
        "libonnxruntime.so",
        "libonnxruntime.dylib",
    ]);

    let mut result = Vec::new();
    let ov_cpu_status = if openvino.is_some() {
        AvailabilityStatus::Ready
    } else {
        AvailabilityStatus::CompatibleMissingRuntime
    };
    result.push(availability(
        InferenceEngine::OpenVino,
        BackendType::Cpu,
        ov_cpu_status,
        openvino.as_ref().map_or_else(
            || "未检测到 OpenVINO Runtime".to_string(),
            |p| format!("检测到 OpenVINO: {}", p.display()),
        ),
    ));

    result.push(availability(
        InferenceEngine::OnnxRuntime,
        BackendType::Cpu,
        if onnx.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "ONNX Runtime CPU 要求本地 onnxruntime 动态库",
    ));

    result.push(availability(
        InferenceEngine::OnnxRuntime,
        BackendType::Cuda,
        if !has_nvidia {
            AvailabilityStatus::Unsupported
        } else if onnx.is_some() && cuda_driver.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "ONNX Runtime CUDA 要求 NVIDIA 驱动和 CUDA 版 ONNX Runtime",
    ));

    result.push(availability(
        InferenceEngine::LlamaCpp,
        BackendType::Cpu,
        if llama.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        llama.as_ref().map_or_else(
            || "未在 PATH 中检测到 llama-cli".to_string(),
            |p| format!("检测到 llama.cpp: {}", p.display()),
        ),
    ));

    let cuda_status = if !has_nvidia {
        AvailabilityStatus::Unsupported
    } else if cuda_driver.is_some() && llama.is_some() {
        AvailabilityStatus::Ready
    } else {
        AvailabilityStatus::CompatibleMissingRuntime
    };
    result.push(availability(
        InferenceEngine::LlamaCpp,
        BackendType::Cuda,
        cuda_status,
        "CUDA 方案要求 NVIDIA 驱动和可用的 llama-cli；具体构建仍需确认启用了 CUDA",
    ));

    result.push(availability(
        InferenceEngine::TensorRT,
        BackendType::TensorRT,
        if !has_nvidia {
            AvailabilityStatus::Unsupported
        } else if cuda_driver.is_some() && tensorrt.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "TensorRT 方案要求 NVIDIA 驱动和 TensorRT Runtime",
    ));

    result.push(availability(
        InferenceEngine::ROCm,
        BackendType::Rocm,
        if !has_amd {
            AvailabilityStatus::Unsupported
        } else if rocm.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "ROCm/HIP 仅在检测到兼容 AMD 硬件和本地 Runtime 后标记为可用",
    ));

    result.push(availability(
        InferenceEngine::DirectML,
        BackendType::DirectML,
        if !is_windows {
            AvailabilityStatus::Unsupported
        } else if has_d3d12 && directml.is_some() {
            AvailabilityStatus::Ready
        } else if has_d3d12 {
            AvailabilityStatus::CompatibleMissingRuntime
        } else {
            AvailabilityStatus::Unsupported
        },
        "DirectML 要求 Windows、D3D12 适配器和 DirectML Runtime",
    ));

    result.push(availability(
        InferenceEngine::OpenVino,
        BackendType::OpenVinoGpu,
        if !has_intel {
            AvailabilityStatus::Unsupported
        } else if openvino.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "OpenVINO GPU 要求 Intel GPU、驱动和 OpenVINO Runtime",
    ));

    result.push(availability(
        InferenceEngine::LlamaCpp,
        BackendType::Metal,
        if !cfg!(target_os = "macos") || !has_apple {
            AvailabilityStatus::Unsupported
        } else if llama.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "Metal 方案要求 macOS Apple GPU 和支持 Metal 的 llama-cli",
    ));

    result.push(availability(
        InferenceEngine::LlamaCpp,
        BackendType::Vulkan,
        if !has_vulkan {
            AvailabilityStatus::Unsupported
        } else if llama.is_some() {
            AvailabilityStatus::Ready
        } else {
            AvailabilityStatus::CompatibleMissingRuntime
        },
        "Vulkan 方案要求 Vulkan 适配器和支持 Vulkan 的 llama-cli",
    ));

    result
}
