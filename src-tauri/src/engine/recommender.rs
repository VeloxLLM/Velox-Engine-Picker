//! 基于硬件信息的推理引擎推荐算法
//!
//! 版本分层：
//! - v1 (default)：仅 Windows，推荐 DirectML / CUDA / OpenVINO
//! - v2          ：+ macOS，推荐 Metal（Apple Silicon）/ AMD64 macOS
//! - v3          ：+ ARM，感知 AArch64 架构（Snapdragon X / 树莓派 / Apple Silicon）

use crate::engine::types::{
    AvailabilityStatus, BackendType, EngineBackendPair, EngineRecommendation, InferenceEngine,
};
use crate::hardware::{CpuArch, GpuType, GpuVendor, HardwareInfo};

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
    cfg!(target_os = "windows")
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

    let nvidia_discrete = hw
        .gpus
        .iter()
        .find(|g| g.vendor == GpuVendor::Nvidia && g.gpu_type == GpuType::Discrete);
    let amd_discrete = hw
        .gpus
        .iter()
        .find(|g| g.vendor == GpuVendor::Amd && g.gpu_type == GpuType::Discrete);
    let intel_gpu = hw.gpus.iter().find(|g| g.vendor == GpuVendor::Intel);
    let apple_gpu = if metal_allowed() {
        hw.gpus.iter().find(|g| g.vendor == GpuVendor::Apple)
    } else {
        None
    };

    let primary: EngineBackendPair;

    if let Some(g) = nvidia_discrete {
        primary = match g.vram {
            Some(vram) if vram >= NVIDIA_MIN_VRAM_MB_FOR_7B => {
                reasons.push(format!(
                    "检测到 NVIDIA 独立显卡「{}」，专用显存 {} MB；理论上优先 TensorRT",
                    g.name, vram
                ));
                EngineBackendPair::new(InferenceEngine::TensorRT, BackendType::TensorRT)
            }
            Some(vram) => {
                reasons.push(format!(
                    "检测到 NVIDIA 独立显卡「{}」，专用显存 {} MB；优先选择开销较低的 llama.cpp CUDA",
                    g.name, vram
                ));
                EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cuda)
            }
            None => {
                reasons.push(format!(
                    "检测到 NVIDIA 独立显卡「{}」，但无法读取专用显存；暂以兼容性较广的 llama.cpp CUDA 为保守建议，不推断显存不足",
                    g.name
                ));
                EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cuda)
            }
        };
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::LlamaCpp,
            BackendType::Cuda,
        ));
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::OnnxRuntime,
            BackendType::Cuda,
        ));
        if directml_allowed() {
            alternatives.push(EngineBackendPair::new(
                InferenceEngine::DirectML,
                BackendType::DirectML,
            ));
        }
    } else if let Some(g) = amd_discrete {
        if directml_allowed() {
            reasons.push(format!(
                "检测到 AMD 独立显卡「{}」；Windows 首选 DirectML，ROCm 仅在检测到 HIP Runtime 时作为备选",
                g.name
            ));
            primary = EngineBackendPair::new(InferenceEngine::DirectML, BackendType::DirectML);
            alternatives.push(EngineBackendPair::new(
                InferenceEngine::ROCm,
                BackendType::Rocm,
            ));
        } else {
            reasons.push(format!("检测到 AMD 独立显卡「{}」，理论首选 ROCm", g.name));
            primary = EngineBackendPair::new(InferenceEngine::ROCm, BackendType::Rocm);
        }
        if directml_allowed() && primary.backend != BackendType::DirectML {
            alternatives.push(EngineBackendPair::new(
                InferenceEngine::DirectML,
                BackendType::DirectML,
            ));
        }
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::LlamaCpp,
            BackendType::Vulkan,
        ));
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
            alternatives.push(EngineBackendPair::new(
                InferenceEngine::DirectML,
                BackendType::DirectML,
            ));
        }
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::OpenVino,
            BackendType::Cpu,
        ));
    } else if let Some(g) = apple_gpu {
        reasons.push(format!(
            "检测到 Apple 芯片 GPU「{}」，推荐 llama.cpp + Metal 后端（苹果生态最成熟方案）",
            g.name
        ));
        primary = EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Metal);
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::OnnxRuntime,
            BackendType::Cpu,
        ));
    } else {
        if arm_aware() && hw.cpu.arch.is_arm() {
            reasons.push(
                "未检测到可用 GPU 加速器，且为 ARM 架构 CPU，优先推荐 llama.cpp (ARM64 + NEON 编译优化)"
                    .to_string(),
            );
            primary = EngineBackendPair::new(InferenceEngine::LlamaCpp, BackendType::Cpu);
        } else {
            reasons.push(
                "未检测到可用 GPU 加速器，默认使用 OpenVINO CPU 推理方案（x86_64 AVX2/AVX-512 优化）".to_string(),
            );
            primary = EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::Cpu);
        }
    }

    if !alternatives.iter().any(|a| a.backend == BackendType::Cpu) {
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::LlamaCpp,
            BackendType::Cpu,
        ));
        alternatives.push(EngineBackendPair::new(
            InferenceEngine::OpenVino,
            BackendType::Cpu,
        ));
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
    let mut ordered_candidates = std::iter::once(primary).chain(dedup_alternatives.iter().copied());
    let ready_primary = ordered_candidates.find(|candidate| {
        hw.availability.iter().any(|availability| {
            availability.target == *candidate && availability.status == AvailabilityStatus::Ready
        })
    });
    let mut warnings = hw.detection_warnings.clone();
    if ready_primary.is_none() {
        warnings.push(
            "未检测到可直接运行的本地推理 Runtime；下方首推是硬件兼容性建议，并非已安装验证"
                .to_string(),
        );
    }

    EngineRecommendation {
        primary,
        theoretical_primary: primary,
        ready_primary,
        alternatives: dedup_alternatives,
        reasons,
        warnings,
        memory_tip,
        session_id: uuid::Uuid::new_v4().to_string(),
        session_ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
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
            "系统内存 {} GB；实际可用模型规模还取决于量化、上下文长度和运行时开销",
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
                0..=3 => format!(
                    "「{}」显存较小（{} GB），建议 7B Q4_K_M 或更小模型",
                    g.name, vram_gb
                ),
                4..=7 => format!("「{}」显存 {} GB，通常适合较小的量化模型", g.name, vram_gb),
                8..=15 => format!("「{}」显存 {} GB，可评估中等规模量化模型", g.name, vram_gb),
                16..=23 => format!("「{}」显存 {} GB，可评估较大规模量化模型", g.name, vram_gb),
                _ => format!(
                    "「{}」显存 {} GB；请按模型文件、KV Cache 与后端开销核算",
                    g.name, vram_gb
                ),
            };
            tips.push(note);
        } else {
            tips.push(format!("无法读取「{}」的专用显存，未估算模型规模", g.name));
        }
    }

    if tips.is_empty() {
        None
    } else {
        Some(tips.join("；"))
    }
}

/// 合并多次检测的推荐结果
///
/// 规则：
/// - primary 取最新一次的结果
/// - alternatives 去重合并（按出现顺序保留首次出现的）
/// - reasons 去重合并（按出现顺序保留首次出现的）
/// - memory_tip 取最新一次的结果
/// - session_id 重新生成，session_ts 取最新一次的时间戳
#[must_use]
pub fn merge_recommendations(recs: &[EngineRecommendation]) -> Option<EngineRecommendation> {
    if recs.is_empty() {
        return None;
    }

    let latest = recs.last().unwrap();

    // 去重合并 alternatives
    let mut seen_pairs: Vec<EngineBackendPair> = Vec::new();
    for r in recs {
        for alt in &r.alternatives {
            if !seen_pairs.contains(alt) {
                seen_pairs.push(*alt);
            }
        }
    }

    // 去重合并 reasons
    let mut seen_reasons: Vec<String> = Vec::new();
    for r in recs {
        for reason in &r.reasons {
            if !seen_reasons.contains(reason) {
                seen_reasons.push(reason.clone());
            }
        }
    }

    Some(EngineRecommendation {
        primary: latest.primary,
        theoretical_primary: latest.theoretical_primary,
        ready_primary: latest.ready_primary,
        alternatives: seen_pairs,
        reasons: seen_reasons,
        warnings: latest.warnings.clone(),
        memory_tip: latest.memory_tip.clone(),
        session_id: uuid::Uuid::new_v4().to_string(),
        session_ts: latest.session_ts,
    })
}

#[cfg(test)]
mod tests {
    use crate::engine::types::{
        AvailabilityStatus, BackendType, EngineAvailability, EngineBackendPair, InferenceEngine,
    };
    use crate::hardware::{
        CpuArch, CpuBrand, CpuInfo, GpuBackend, GpuInfo, GpuType, GpuVendor, HardwareInfo,
        MemoryInfo, PlatformInfo,
    };

    fn make_x86_cpu() -> CpuInfo {
        CpuInfo {
            name: "Intel Core i7-13700K".into(),
            vendor: "GenuineIntel".into(),
            core_count: 16,
            logical_processor_count: 24,
            frequency: 3400,
            brand: CpuBrand::Intel,
            arch: CpuArch::X86_64,
            instruction_sets: vec!["AVX2".into()],
        }
    }

    fn make_hw(cpu: CpuInfo, gpus: Vec<GpuInfo>, total_mb: u64) -> HardwareInfo {
        HardwareInfo {
            cpu,
            memory: MemoryInfo {
                total: total_mb,
                available: total_mb / 2,
            },
            gpus,
            platform: PlatformInfo {
                os: std::env::consts::OS.into(),
                arch: std::env::consts::ARCH.into(),
                edition: "v1".into(),
                support_level: "Stable".into(),
            },
            availability: vec![],
            detection_warnings: vec![],
        }
    }

    /// 1. NVIDIA dGPU 显存充足（≥4GB）→ 首推引擎应为 TensorRT
    #[test]
    fn test_nvidia_vram_sufficient_recommends_tensorrt() {
        let hw = make_hw(
            make_x86_cpu(),
            vec![GpuInfo {
                name: "NVIDIA GeForce RTX 4060".into(),
                vendor: GpuVendor::Nvidia,
                gpu_type: GpuType::Discrete,
                vram: Some(6 * 1024),
                backend: GpuBackend::Vulkan,
                device_id: (0x10DE, 0),
            }],
            16 * 1024,
        );
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::TensorRT);
        assert_eq!(rec.primary.backend, BackendType::TensorRT);
    }

    /// 2. NVIDIA dGPU 显存不足（<4GB）→ 首推引擎应为 LlamaCpp，后端为 Cuda
    #[test]
    fn test_nvidia_vram_insufficient_recommends_llamacpp_cuda() {
        let hw = make_hw(
            make_x86_cpu(),
            vec![GpuInfo {
                name: "NVIDIA GeForce GT 1030".into(),
                vendor: GpuVendor::Nvidia,
                gpu_type: GpuType::Discrete,
                vram: Some(2 * 1024),
                backend: GpuBackend::Vulkan,
                device_id: (0x10DE, 0),
            }],
            16 * 1024,
        );
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::LlamaCpp);
        assert_eq!(rec.primary.backend, BackendType::Cuda);
    }

    #[test]
    fn test_nvidia_unknown_vram_is_not_reported_as_low_vram() {
        let hw = make_hw(
            make_x86_cpu(),
            vec![GpuInfo {
                name: "NVIDIA Test Adapter".into(),
                vendor: GpuVendor::Nvidia,
                gpu_type: GpuType::Discrete,
                vram: None,
                backend: GpuBackend::Dx12,
                device_id: (0x10DE, 0),
            }],
            16 * 1024,
        );
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.backend, BackendType::Cuda);
        assert!(rec
            .reasons
            .iter()
            .any(|reason| reason.contains("不推断显存不足")));
    }

    #[test]
    fn test_ready_primary_requires_runtime_evidence() {
        let mut hw = make_hw(make_x86_cpu(), vec![], 16 * 1024);
        hw.availability.push(EngineAvailability {
            target: EngineBackendPair::new(InferenceEngine::OpenVino, BackendType::Cpu),
            status: AvailabilityStatus::Ready,
            version: None,
            evidence: "test fixture".into(),
        });
        let rec = super::recommend(&hw);
        assert_eq!(rec.ready_primary, Some(rec.primary));
        assert!(rec.warnings.is_empty());
    }

    /// 3. Intel iGPU → 首推引擎应为 OpenVino，后端为 OpenVinoGpu
    #[test]
    fn test_intel_igpu_recommends_openvino_gpu() {
        let hw = make_hw(
            make_x86_cpu(),
            vec![GpuInfo {
                name: "Intel UHD Graphics 770".into(),
                vendor: GpuVendor::Intel,
                gpu_type: GpuType::Integrated,
                vram: None,
                backend: GpuBackend::Vulkan,
                device_id: (0x8086, 0),
            }],
            16 * 1024,
        );
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::OpenVino);
        assert_eq!(rec.primary.backend, BackendType::OpenVinoGpu);
    }

    /// 4. AMD dGPU → 首推引擎应为 ROCm
    #[test]
    fn test_amd_dgpu_recommendation_is_platform_aware() {
        let hw = make_hw(
            make_x86_cpu(),
            vec![GpuInfo {
                name: "AMD Radeon RX 7900 XTX".into(),
                vendor: GpuVendor::Amd,
                gpu_type: GpuType::Discrete,
                vram: Some(24 * 1024),
                backend: GpuBackend::Vulkan,
                device_id: (0x1002, 0),
            }],
            32 * 1024,
        );
        let rec = super::recommend(&hw);
        if cfg!(target_os = "windows") {
            assert_eq!(rec.primary.engine, InferenceEngine::DirectML);
            assert_eq!(rec.primary.backend, BackendType::DirectML);
        } else {
            assert_eq!(rec.primary.engine, InferenceEngine::ROCm);
            assert_eq!(rec.primary.backend, BackendType::Rocm);
        }
    }

    /// 5. Apple Silicon（需要 feature v2 或 v3）→ 首推引擎应为 LlamaCpp，后端为 Metal
    #[cfg(any(feature = "v2", feature = "v3"))]
    #[test]
    fn test_apple_silicon_recommends_llamacpp_metal() {
        let hw = HardwareInfo {
            cpu: CpuInfo {
                name: "Apple M3 Pro".into(),
                vendor: "Apple".into(),
                core_count: 12,
                logical_processor_count: 12,
                frequency: 0,
                brand: CpuBrand::Apple,
                arch: CpuArch::Aarch64,
                instruction_sets: vec!["NEON".into()],
            },
            memory: MemoryInfo {
                total: 18 * 1024,
                available: 12 * 1024,
            },
            gpus: vec![GpuInfo {
                name: "Apple M3 Pro".into(),
                vendor: GpuVendor::Apple,
                gpu_type: GpuType::Integrated,
                vram: None,
                backend: GpuBackend::Metal,
                device_id: (0x106B, 0),
            }],
            platform: PlatformInfo {
                os: "macos".into(),
                arch: "aarch64".into(),
                edition: "v2".into(),
                support_level: "Experimental".into(),
            },
            availability: vec![],
            detection_warnings: vec![],
        };
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::LlamaCpp);
        assert_eq!(rec.primary.backend, BackendType::Metal);
    }

    /// 6. 纯 CPU x86_64（无 GPU）→ 首推引擎应为 OpenVino，后端为 Cpu
    #[test]
    fn test_cpu_only_x86_64_recommends_openvino_cpu() {
        let hw = make_hw(make_x86_cpu(), vec![], 16 * 1024);
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::OpenVino);
        assert_eq!(rec.primary.backend, BackendType::Cpu);
    }

    /// 7. ARM CPU 无 GPU（需要 feature v3）→ 首推引擎应为 LlamaCpp，后端为 Cpu
    #[cfg(feature = "v3")]
    #[test]
    fn test_arm_cpu_no_gpu_recommends_llamacpp_cpu() {
        let hw = HardwareInfo {
            cpu: CpuInfo {
                name: "Qualcomm Snapdragon X Elite".into(),
                vendor: "Qualcomm".into(),
                core_count: 12,
                logical_processor_count: 12,
                frequency: 0,
                brand: CpuBrand::Qualcomm,
                arch: CpuArch::Aarch64,
                instruction_sets: vec!["NEON".into()],
            },
            memory: MemoryInfo {
                total: 16 * 1024,
                available: 8 * 1024,
            },
            gpus: vec![],
            platform: PlatformInfo {
                os: "windows".into(),
                arch: "aarch64".into(),
                edition: "v3".into(),
                support_level: "Experimental".into(),
            },
            availability: vec![],
            detection_warnings: vec![],
        };
        let rec = super::recommend(&hw);
        assert_eq!(rec.primary.engine, InferenceEngine::LlamaCpp);
        assert_eq!(rec.primary.backend, BackendType::Cpu);
    }
}
