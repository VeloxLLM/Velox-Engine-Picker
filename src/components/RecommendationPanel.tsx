import { useState } from "react";
import type {
  HardwareInfo,
  EngineRecommendation,
  InferenceEngine,
  BackendType,
} from "../types";
import {
  ENGINE_NAMES,
  BACKEND_NAMES,
  ENGINE_VENDORS,
  ENGINE_DESCRIPTIONS,
} from "../types";

interface Props {
  hw: HardwareInfo;
  rec: EngineRecommendation;
}

const BACKEND_COLORS: Record<BackendType, string> = {
  Cpu: "#3b82f6",
  OpenVinoGpu: "#0071c5",
  Cuda: "#76b900",
  TensorRT: "#76b900",
  Rocm: "#ed1c24",
  DirectML: "#f97316",
  Metal: "#a8a8a8",
  Vulkan: "#a855f7",
};

const BACKEND_CATEGORIES: Record<BackendType, string> = {
  Cpu: "CPU",
  OpenVinoGpu: "GPU (通用)",
  Cuda: "GPU (NVIDIA)",
  TensorRT: "GPU (NVIDIA)",
  Rocm: "GPU (AMD)",
  DirectML: "GPU (通用)",
  Metal: "GPU (通用)",
  Vulkan: "GPU (通用)",
};

export default function RecommendationPanel({ hw, rec }: Props) {
  const [cheatOpen, setCheatOpen] = useState(false);

  const primaryColor = BACKEND_COLORS[rec.primary.backend];

  // 速查表数据
  const cheatData: [InferenceEngine, BackendType, string, boolean][] = [
    ["OpenVino", "Cpu", "Intel/AMD CPU 通用，适合纯 CPU 服务器", true],
    ["OpenVino", "OpenVinoGpu", "Intel iGPU / Arc 独显，官方优化", hw.gpus.some(g => g.vendor === "Intel")],
    ["LlamaCpp", "Cpu", "本地轻量推理首选，支持 GGUF 量化", true],
    ["LlamaCpp", "Cuda", "NVIDIA GPU，需要 cuBLAS 编译", hw.gpus.some(g => g.vendor === "Nvidia" && g.gpu_type === "Discrete")],
    ["TensorRT", "TensorRT", "NVIDIA 最高性能，INT8/FP8 量化强", hw.gpus.some(g => g.vendor === "Nvidia" && g.gpu_type === "Discrete")],
    ["ROCm", "Rocm", "AMD dGPU / APU，Linux 生态优先", hw.gpus.some(g => g.vendor === "Amd" && g.gpu_type === "Discrete")],
    ["DirectML", "DirectML", "Windows 通用，兼容 Intel/AMD/NVIDIA", true],
  ];

  return (
    <div>
      <h2 style={{ fontSize: 20, marginBottom: 4 }}>推荐推理引擎</h2>
      <p style={{ color: "#888", fontSize: 12, marginBottom: 16 }}>
        基于本机硬件自动匹配
      </p>

      {/* Primary Card */}
      <div className="primary-card" style={{ borderColor: primaryColor }}>
        <div className="primary-badge" style={{ color: primaryColor }}>
          ⭐ 首推方案 (Recommended)
        </div>
        <div className="primary-row">
          <span className="primary-engine">
            {ENGINE_NAMES[rec.primary.engine]}
          </span>
          <span className="primary-sep" />
          <span className="primary-backend" style={{ color: primaryColor }}>
            {BACKEND_NAMES[rec.primary.backend]}
          </span>
        </div>
        <div style={{ display: "flex", gap: 8, marginBottom: 10 }}>
          <span className="chip">厂商：{ENGINE_VENDORS[rec.primary.engine]}</span>
          <span className="chip">
            后端类型：{BACKEND_CATEGORIES[rec.primary.backend]}
          </span>
        </div>
        <div className="primary-desc">
          {ENGINE_DESCRIPTIONS[rec.primary.engine]}
        </div>
      </div>

      {/* Reasons */}
      {rec.reasons.length > 0 && (
        <div className="card">
          <div className="card-title">推荐理由</div>
          <ul style={{ listStyle: "none", padding: 0 }}>
            {rec.reasons.map((r, i) => (
              <li
                key={i}
                style={{
                  display: "flex",
                  gap: 8,
                  marginBottom: 4,
                  fontSize: 13,
                }}
              >
                <span style={{ color: "#10b981" }}>•</span>
                <span>{r}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Alternatives */}
      <div className="card">
        <div className="card-title">备选方案（按优先级降序）</div>
        {rec.alternatives.length === 0 ? (
          <span style={{ color: "#888" }}>（无）</span>
        ) : (
          <div className="table-grid">
            <div className="table-row header">
              <span>#</span>
              <span>引擎</span>
              <span>后端</span>
              <span>厂商</span>
            </div>
            {rec.alternatives.map((a, i) => {
              const color = BACKEND_COLORS[a.backend];
              return (
                <div className="table-row striped" key={i}>
                  <span style={{ color, fontWeight: 600 }}>{i + 1}</span>
                  <span style={{ fontWeight: 600 }}>{ENGINE_NAMES[a.engine]}</span>
                  <span style={{ color }}>{BACKEND_NAMES[a.backend]}</span>
                  <span style={{ color: "#888" }}>{ENGINE_VENDORS[a.engine]}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Memory Tip */}
      {rec.memory_tip && (
        <div className="memory-tip">
          <strong>模型大小建议</strong>
          {rec.memory_tip}
        </div>
      )}

      {/* Cheat Sheet */}
      <details
        className="cheat-sheet card"
        open={cheatOpen}
        onToggle={(e) => setCheatOpen(e.currentTarget.open)}
      >
        <summary>引擎 / 后端速查表</summary>
        <div className="table-grid">
          <div className="table-row header">
            <span>引擎</span>
            <span>后端</span>
            <span>适用场景</span>
            <span>本机可用</span>
          </div>
          {cheatData.map(([eng, backend, scene, ok], i) => {
            const color = BACKEND_COLORS[backend];
            return (
              <div className="table-row striped" key={i}>
                <span style={{ fontWeight: 600 }}>{ENGINE_NAMES[eng]}</span>
                <span style={{ color }}>{BACKEND_NAMES[backend]}</span>
                <span style={{ color: "#888" }}>{scene}</span>
                <span style={{ color: ok ? "#10b981" : "#666" }}>
                  {ok ? "是" : "否"}
                </span>
              </div>
            );
          })}
        </div>
      </details>
    </div>
  );
}