// TypeScript 类型定义 — 与 Rust 侧 serde 类型对齐

export type CpuArch = "X86_64" | "Aarch64" | "Arm" | "X86" | "Other";
export type CpuBrand = "Intel" | "Amd" | "Apple" | "Qualcomm" | "Other";
export type GpuVendor = "Nvidia" | "Amd" | "Intel" | "Apple" | "Qualcomm" | "Microsoft" | "Other";
export type GpuType = "Discrete" | "Integrated" | "Other";
export type GpuBackend = "Vulkan" | "Metal" | "Dx12" | "Gl" | "BrowserWebGpu" | "Other";

export type InferenceEngine =
  | "OpenVino"
  | "LlamaCpp"
  | "OnnxRuntime"
  | "TensorRT"
  | "ROCm"
  | "DirectML";

export type BackendType =
  | "Cpu"
  | "OpenVinoGpu"
  | "Cuda"
  | "TensorRT"
  | "Rocm"
  | "DirectML"
  | "Metal"
  | "Vulkan";

export interface CpuInfo {
  name: string;
  vendor: string;
  core_count: number;
  logical_processor_count: number;
  frequency: number;
  brand: CpuBrand;
  arch: CpuArch;
  instruction_sets: string[];
}

export interface MemoryInfo {
  total: number;
  available: number;
}

export interface GpuInfo {
  name: string;
  vendor: GpuVendor;
  gpu_type: GpuType;
  vram: number | null;
  backend: GpuBackend;
  device_id: [number, number];
}

export interface HardwareInfo {
  cpu: CpuInfo;
  memory: MemoryInfo;
  gpus: GpuInfo[];
  platform: PlatformInfo;
  availability: EngineAvailability[];
  detection_warnings: string[];
}

export interface PlatformInfo {
  os: string;
  arch: string;
  edition: string;
  support_level: "Stable" | "Experimental" | string;
}

export type AvailabilityStatus =
  | "Ready"
  | "CompatibleMissingRuntime"
  | "Unsupported"
  | "Unknown";

export interface EngineAvailability {
  target: EngineBackendPair;
  status: AvailabilityStatus;
  version: string | null;
  evidence: string;
}

export interface EngineBackendPair {
  engine: InferenceEngine;
  backend: BackendType;
}

export interface EngineRecommendation {
  primary: EngineBackendPair;
  theoretical_primary: EngineBackendPair;
  ready_primary: EngineBackendPair | null;
  alternatives: EngineBackendPair[];
  reasons: string[];
  warnings: string[];
  memory_tip: string | null;
  session_id: string;
  session_ts: number;
}

// 辅助函数：引擎/后端显示名
export const ENGINE_NAMES: Record<InferenceEngine, string> = {
  OpenVino: "OpenVINO",
  LlamaCpp: "llama.cpp",
  OnnxRuntime: "ONNX Runtime",
  TensorRT: "TensorRT",
  ROCm: "ROCm",
  DirectML: "DirectML",
};

export const BACKEND_NAMES: Record<BackendType, string> = {
  Cpu: "CPU",
  OpenVinoGpu: "OpenVINO GPU",
  Cuda: "CUDA",
  TensorRT: "TensorRT",
  Rocm: "ROCm",
  DirectML: "DirectML",
  Metal: "Metal",
  Vulkan: "Vulkan",
};

export const ENGINE_VENDORS: Record<InferenceEngine, string> = {
  OpenVino: "Intel",
  LlamaCpp: "ggerganov (Open Source)",
  OnnxRuntime: "Microsoft",
  TensorRT: "NVIDIA",
  ROCm: "AMD",
  DirectML: "Microsoft",
};

export const ENGINE_DESCRIPTIONS: Record<InferenceEngine, string> = {
  OpenVino: "Intel 开源推理引擎，对 Intel CPU / iGPU / Arc 优化最佳",
  LlamaCpp: "社区最流行的本地 LLM 推理框架，支持 CPU 和多种 GPU 后端",
  OnnxRuntime: "微软跨平台 ONNX 推理运行时，硬件支持广泛",
  TensorRT: "NVIDIA 官方深度学习推理优化器，推理速度最快",
  ROCm: "AMD 开源 GPU 计算平台，对标 CUDA",
  DirectML: "Windows DirectX 12 通用 ML 推理后端，兼容 Intel/AMD/NVIDIA",
};
