//! 推理引擎类型定义

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum InferenceEngine {
    OpenVino,
    LlamaCpp,
    OnnxRuntime,
    TensorRT,
    ROCm,
    DirectML,
}

impl From<InferenceEngine> for String {
    fn from(v: InferenceEngine) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for InferenceEngine {
    fn from(s: String) -> Self {
        match s.as_str() {
            "OpenVino" => Self::OpenVino,
            "LlamaCpp" => Self::LlamaCpp,
            "OnnxRuntime" => Self::OnnxRuntime,
            "TensorRT" => Self::TensorRT,
            "ROCm" => Self::ROCm,
            "DirectML" => Self::DirectML,
            _ => Self::OpenVino,
        }
    }
}

impl InferenceEngine {
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::OpenVino => "OpenVINO",
            Self::LlamaCpp => "llama.cpp",
            Self::OnnxRuntime => "ONNX Runtime",
            Self::TensorRT => "TensorRT",
            Self::ROCm => "ROCm",
            Self::DirectML => "DirectML",
        }
    }

    #[must_use]
    pub fn vendor(self) -> &'static str {
        match self {
            Self::OpenVino => "Intel",
            Self::LlamaCpp => "ggerganov (Open Source)",
            Self::OnnxRuntime => "Microsoft",
            Self::TensorRT => "NVIDIA",
            Self::ROCm => "AMD",
            Self::DirectML => "Microsoft",
        }
    }

    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::OpenVino => "Intel 开源推理引擎，对 Intel CPU / iGPU / Arc 优化最佳",
            Self::LlamaCpp => "社区最流行的本地 LLM 推理框架，支持 CPU 和多种 GPU 后端",
            Self::OnnxRuntime => "微软跨平台 ONNX 推理运行时，硬件支持广泛",
            Self::TensorRT => "NVIDIA 官方深度学习推理优化器，推理速度最快",
            Self::ROCm => "AMD 开源 GPU 计算平台，对标 CUDA",
            Self::DirectML => "Windows DirectX 12 通用 ML 推理后端，兼容 Intel/AMD/NVIDIA",
        }
    }
}

impl fmt::Display for InferenceEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum BackendType {
    Cpu,
    OpenVinoGpu,
    Cuda,
    TensorRT,
    Rocm,
    DirectML,
    Metal,
    Vulkan,
}

impl From<BackendType> for String {
    fn from(v: BackendType) -> String {
        format!("{:?}", v)
    }
}

impl From<String> for BackendType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Cpu" => Self::Cpu,
            "OpenVinoGpu" => Self::OpenVinoGpu,
            "Cuda" => Self::Cuda,
            "TensorRT" => Self::TensorRT,
            "Rocm" => Self::Rocm,
            "DirectML" => Self::DirectML,
            "Metal" => Self::Metal,
            "Vulkan" => Self::Vulkan,
            _ => Self::Cpu,
        }
    }
}

impl BackendType {
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::OpenVinoGpu => "OpenVINO GPU",
            Self::Cuda => "CUDA",
            Self::TensorRT => "TensorRT",
            Self::Rocm => "ROCm",
            Self::DirectML => "DirectML",
            Self::Metal => "Metal",
            Self::Vulkan => "Vulkan",
        }
    }

    #[must_use]
    pub fn category(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::OpenVinoGpu | Self::Metal | Self::Vulkan | Self::DirectML => "GPU (通用)",
            Self::Cuda | Self::TensorRT => "GPU (NVIDIA)",
            Self::Rocm => "GPU (AMD)",
        }
    }
}

impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineBackendPair {
    pub engine: InferenceEngine,
    pub backend: BackendType,
}

impl EngineBackendPair {
    #[must_use]
    pub const fn new(engine: InferenceEngine, backend: BackendType) -> Self {
        Self { engine, backend }
    }
}

impl fmt::Display for EngineBackendPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.engine, self.backend)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRecommendation {
    pub primary: EngineBackendPair,
    pub alternatives: Vec<EngineBackendPair>,
    pub reasons: Vec<String>,
    pub memory_tip: Option<String>,
    pub session_id: String,
    pub session_ts: u64,
}