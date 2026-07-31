//! 基于硬件信息的推理引擎推荐算法
//!
//! 版本分层：
//! - v1 (default)：仅 Windows，推荐 DirectML / CUDA / OpenVINO
//! - v2          ：+ macOS，推荐 Metal（Apple Silicon）/ AMD64 macOS
//! - v3          ：+ ARM，感知 AArch64 架构（Snapdragon X / 树莓派 / Apple Silicon）

use crate::hardware::{CpuArch, GpuType, GpuVendor, HardwareInfo};
use crate::engine::types::{BackendType, EngineBackendPair, EngineRecommendation, InferenceEngine};

const NVIDIA_MIN_VRAM_MB_FOR_7B: u64 = 4 * 1024;
const SYSTEM_MIN_RAM_MB_FOR_7B_CPU: u64 = 8 * 1024;

#[allow(clippy::collapsible_else_if)]
fn backend_allowed(b: BackendType) -> bool {
    #[cfg(feature = "v3")]
    {
        let _ = b;
        true
    }
    #[cfg(not(feature = "v3"))]
    {
        #[cfg(feature = "v2")]
        {
            let _ = b;
            true
        }
        #[cfg(not(feature = "v2"))]
        {
            !matches!(b, BackendType::Metal)
        }
    }
}

fn directml_allowed() -> bool {
    #[cfg(target_os = "windows")]
    {
        true
    }
    #[cfg(not(target_os = "windows"))]
    {
        cfg!(any(feature = "v2", feature = "v3"))
    }
}

fn metal_allowed() -> bool {
    cfg!(feature = "v2") || cfg!(feature = "v3")
}

fn arm_aware() -> bool {
    cfg!(feature = "v3")
}

#[must_use]
pub fn recommend(hw: &HardwareInfo) -> EngineRecommendation {
    let mut reasons: Vec<String> = Vec::new();
    let mut alternatives: Vec<EngineBackendPair> = Vec::new();

    if arm_aware() && hw.cpu.arch.is_arm() {
        reasons.push(format!(
            "检测到 ARM 架构 CPU「{} (架构: {})」，将优先使用支持 NEON / ARM64 编译优化的引擎",
            hw.cpu.name,
            hw.cpu.arch.display_name()
        ));
    }

    let nvidia_discrete = hw.gpus.iter().find(|g| {
        g.vendor == GpuVendor::Nvidia && g.gpu_type == GpuType::Discrete
    });
    let amd_discrete = hw.gpus.iter().find(|g| {
        g.vendor == GpuVendor::Amd && g.gpu_type == GpuType::Discrete
    });
    let intel_gpu = hw.gpus.iter().find(|g| g.vendor == GpuVendor::Intel);
    let apple_gpu = if metal_allowed() {
        hw.gpus.iter().find(|g| g.vendor == GpuVendor::Apple)
    } else {
        None
    };

    let primary: EngineBackendPair;

    if let Some(g) = nvidia_discrete {
        let enough_vram = g.vram.unwrap_or(0) >= NVIDIA_MIN_VRAM_MB_FOR_7B;
        primary = if enough_vram {
            reasons.push(format!(
                "检测到 NVIDIA 独立显卡「{}」，显存 {} MB，推荐 TensorRT 以获得最佳推理性能",
                g.name,
                g.vram.map(|v| v.to_string()).unwrap_or_else(|| "充足".to_string())
            ));
            EngineBackendPair::new(InferenceEngine::TensorRT, BackendType::TensorRT)
        } else {
            reasons.push(format!(
                "检测到 NVIDIA 独立显卡「{}」，显存较小或未知，推荐 CUDA 后端（兼容性更广泛）",
                g.name
            ));
            EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cuda)
        };
        alternatives.push(EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cuda));
        alternatives.push(EngineBackendPair::new(InferenceEngine::OnnxRuntime, BackendType::Cuda));
        if directml_allowed() {
            alternatives.push(EngineBackendPair::new(InferenceEngine::DirectML, BackendType::DirectML));
        }
    } else if let Some(g) = amd_discrete {
        reasons.push(format!(
            "检测到 AMD 独立显卡「{}」，推荐使用 ROCm 后端（Linux）或 DirectML（Windows）进行 GPU 加速",
            g.name
        ));
        primary = EngineBackendPair::new(InferenceEngine::ROCm, BackendType::Rocm);
        if directml_allowed() {
            alternatives.push(EngineBackendPair::new(InferenceEngine::DirectML, BackendType::DirectML));
        }
        alternatives.push(EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Vulkan));
    } else if let Some(g) = intel_gpu {
        let is_arc = g.name.to_ascii_lowercase().contains("arc") || g.gpu_type == GpuType::Discrete;
        primary = EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::OpenVinoGpu);
        if is_arc {
            reasons.push(format!(
                "检测到 Intel Arc 独立显卡「{}」，推荐 OpenVINO GPU 后端（官方深度优化）",
                g.name
            ));
        } else {
            reasons.push(format!(
                "检测到 Intel 集成显卡「{}」，推荐 OpenVINO GPU 后端，可显著加速 CPU-only 推理",
                g.name
            ));
        }
        if directml_allowed() {
            alternatives.push(EngineBackendPair::new(InferenceEngine::DirectML, BackendType::DirectML));
        }
        alternatives.push(EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::Cpu));
    } else if let Some(g) = apple_gpu {
        reasons.push(format!(
            "检测到 Apple 芯片 GPU「{}」，推荐 llama.cpp + Metal 后端（苹果生态最成熟方案）",
            g.name
        ));
        primary = EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Metal);
        alternatives.push(EngineBackendPair::new(InferenceEngine::OnnxRuntime, BackendType::Cpu));
    } else {
        if arm_aware() && hw.cpu.arch.is_arm() {
            reasons.push(format!(
                "未检测到可用 GPU 加速器，且为 ARM 架构 CPU，优先推荐 llama.cpp (ARM64 + NEON 编译优化)"
            ));
            primary = EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cpu);
        } else {
            reasons.push(
                "未检测到可用 GPU 加速器，默认使用 OpenVINO CPU 推理方案（x86_64 AVX2/AVX-512 优化）".to_string(),
            );
            primary = EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::Cpu);
        }
    }

    if !alternatives.iter().any(|a| a.backend == BackendType::Cpu) {
        alternatives.push(EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cpu));
        alternatives.push(EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::Cpu));
    }

    let mut alternatives: Vec<EngineBackendPair> = alternatives
        .into_iter()
        .filter(|a| backend_allowed(a.backend))
        .collect();

    alternatives.retain(|a| *a != primary);

    let mut seen: Vec<EngineBackendPair> = Vec::with_capacity(alternatives.len());
    let mut dedup_alternatives = Vec::with_capacity(alternatives.len());
    for a in alternatives {
        if !seen.contains(&a) {
            seen.push(a);
            dedup_alternatives.push(a);
        }
    }

    let memory_tip = build_memory_tip(hw);

    EngineRecommendation {
        primary,
        alternatives: dedup_alternatives,
        reasons,
        memory_tip,
    }
}

fn build_memory_tip(hw: &HardwareInfo) -> Option<String> {
    let mut tips = Vec::new();

    if arm_aware() && hw.cpu.arch == CpuArch::Aarch64 {
        tips.push(
            "AArch64 架构：编译 llama.cpp 时请启用 NEON（默认开启）；大内存 SBC 可尝试启用 16-bit 量化".to_string()
        );
    }

    if hw.memory.total < SYSTEM_MIN_RAM_MB_FOR_7B_CPU {
        tips.push(format!(
            "系统内存 {} GB 偏小，CPU 模式建议运行 3B 及更小的量化模型（Q4 或更低）",
            hw.memory.total / 1024
        ));
    } else {
        tips.push(format!(
            "系统内存 {} GB，CPU 模式可流畅运行 7B Q4 量化模型，16 GB+ 可尝试 13B",
            hw.memory.total / 1024
        ));
    }

    for g in &hw.gpus {
        if g.gpu_type != GpuType::Discrete {
            continue;
        }
        if let Some(vram_mb) = g.vram {
            let vram_gb = vram_mb / 1024;
            let note = match vram_gb {
                0..=3 => format!("「{}」显存较小（{} GB），建议 7B Q4_K_M 或更小模型", g.name, vram_gb),
                4..=7 => format!("「{}」显存 {} GB，可运行 7B Q4~Q8 / 13B Q4", g.name, vram_gb),
                8..=15 => format!("「{}」显存 {} GB，可运行 13B / 34B Q4 量化模型", g.name, vram_gb),
                16..=23 => format!("「{}」显存 {} GB，可流畅运行 34B Q4 或 70B Q3", g.name, vram_gb),
                _ => format!("「{}」显存 {} GB，可尝试 70B+ 大模型", g.name, vram_gb),
            };
            tips.push(note);
        }
    }

    if tips.is_empty() {
        None
    } else {
        Some(tips.join("；"))
    }
}