import { memo, useMemo } from "react";
import type { HardwareInfo, GpuVendor, ModelFit } from "../types";

interface Props {
  hw: HardwareInfo;
}

const VENDOR_COLORS: Record<GpuVendor, string> = {
  Nvidia: "#76b900",
  Amd: "#ed1c24",
  Intel: "#0071c5",
  Apple: "#a8a8a8",
  Qualcomm: "#3256d6",
  Microsoft: "#888",
  Other: "#888",
};

const VENDOR_LABELS: Record<GpuVendor, string> = {
  Nvidia: "NVIDIA",
  Amd: "AMD",
  Intel: "Intel",
  Apple: "Apple",
  Qualcomm: "Qualcomm",
  Microsoft: "Microsoft",
  Other: "Unknown",
};

const BRAND_LABELS: Record<string, string> = {
  Intel: "Intel",
  Amd: "AMD",
  Apple: "Apple Silicon",
  Qualcomm: "Qualcomm",
  Other: "其他",
};

// ── 模块级样式常量 ──
const HEADING_STYLE = { fontSize: 20, marginBottom: 4 } as const;
const SUBTITLE_STYLE = { color: "#888", fontSize: 12, marginBottom: 16 } as const;
const SECTION_TITLE_STYLE = { fontSize: 17, marginBottom: 10 } as const;
const FLEX_WRAP_STYLE = { display: "flex", flexWrap: "wrap", gap: 6 } as const;
const FLEX_GAP16_STYLE = { display: "flex", gap: 16, marginBottom: 8 } as const;
const MEM_LABEL_STYLE = { fontWeight: 600 } as const;
const MEM_HINT_STYLE = { fontSize: 12, color: "#888" } as const;
const GPU_NAME_STYLE = { fontWeight: 600, fontSize: 15, marginBottom: 6 } as const;
const GPU_META_STYLE = { fontSize: 11, color: "#666", marginTop: 4 } as const;
const NO_GPU_TEXT_STYLE = { fontSize: 13, color: "#ccc", whiteSpace: "pre-line" } as const;
const FIT_LABELS: Record<ModelFit, string> = {
  Gpu: "适合 GPU",
  Hybrid: "可尝试 GPU + 内存",
  Cpu: "可尝试 CPU",
  InsufficientMemory: "内存可能不足",
  Unknown: "无法估算",
};

function HardwarePanelInner({ hw }: Props) {
  const memTotalGB = useMemo(() => (hw.memory.total / 1024).toFixed(1), [hw.memory.total]);
  const memAvailGB = useMemo(() => (hw.memory.available / 1024).toFixed(1), [hw.memory.available]);

  const memUsedPct = useMemo(
    () => Math.min(100, (1 - hw.memory.available / Math.max(hw.memory.total, 1)) * 100),
    [hw.memory.total, hw.memory.available],
  );

  const memColor = useMemo(
    () => (memUsedPct > 85 ? "#ef4444" : memUsedPct > 60 ? "#f59e0b" : "#10b981"),
    [memUsedPct],
  );
  const discreteGpus = useMemo(() => hw.gpus.filter((gpu) => gpu.gpu_type === "Discrete"), [hw.gpus]);
  const integratedGpus = useMemo(() => hw.gpus.filter((gpu) => gpu.gpu_type === "Integrated"), [hw.gpus]);

  return (
    <div>
      <h2 style={HEADING_STYLE}>硬件配置</h2>
      <p style={SUBTITLE_STYLE}>以下信息由 sysinfo + 平台原生 GPU API 实时采集</p>

      {hw.detection_warnings.map((warning) => (
        <div className="warning-banner" role="status" key={warning}>{warning}</div>
      ))}

      {/* CPU */}
      <div className="card card-accent" style={{ borderLeftColor: "#3b82f6" }}>
        <div className="card-title">CPU</div>
        <div style={GPU_NAME_STYLE}>{hw.cpu.name}</div>
        <div style={FLEX_WRAP_STYLE}>
          <span className="chip">厂商：{hw.cpu.vendor}</span>
          <span className="chip">核心数：{hw.cpu.core_count}</span>
          <span className="chip">逻辑处理器：{hw.cpu.logical_processor_count}</span>
          <span className="chip">频率：{hw.cpu.frequency} MHz</span>
          <span className="chip">架构：{hw.cpu.arch}</span>
          <span className="chip">品牌：{BRAND_LABELS[hw.cpu.brand]}</span>
          {hw.cpu.instruction_sets.map((feature) => (
            <span className="chip" key={feature}>{feature}</span>
          ))}
        </div>
      </div>

      {/* Memory */}
      <div className="card card-accent" style={{ borderLeftColor: "#10b981" }}>
        <div className="card-title">内存 (RAM)</div>
        <div style={FLEX_GAP16_STYLE}>
          <span style={MEM_LABEL_STYLE}>总量：{memTotalGB} GB</span>
          <span>可用：{memAvailGB} GB</span>
        </div>
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{ width: `${memUsedPct}%`, background: memColor }}
          />
        </div>
        <span style={MEM_HINT_STYLE}>使用率：{memUsedPct.toFixed(1)}%</span>
      </div>

      {/* GPU list */}
      <h3 style={SECTION_TITLE_STYLE}>显卡 / GPU（实体设备 {hw.gpus.length} 个）</h3>

      <div className="gpu-summary-grid">
        <div className="card compact-card">
          <div className="card-title">独立显卡（dGPU）</div>
          {discreteGpus.length > 0 ? discreteGpus.map((gpu) => <div key={`${gpu.device_id}-${gpu.name}`}>{gpu.name}</div>) : <span className="muted">未检测到独立显卡</span>}
        </div>
        <div className="card compact-card">
          <div className="card-title">集成显卡（iGPU）</div>
          {integratedGpus.length > 0 ? integratedGpus.map((gpu) => <div key={`${gpu.device_id}-${gpu.name}`}>{gpu.name}</div>) : <span className="muted">未检测到 iGPU（可能未安装驱动或已在 BIOS 中禁用）</span>}
        </div>
      </div>

      {hw.gpus.length === 0 ? (
        <div className="card" style={{ borderLeftColor: "#f59e0b" }}>
          <div className="card-title">未检测到 GPU</div>
          <p style={NO_GPU_TEXT_STYLE}>
            wgpu 未枚举到任何图形适配器。可能原因：
            {"\n"}· 当前环境为服务器/虚拟机，无图形驱动
            {"\n"}· 图形后端（Vulkan/DX12）不可用
            {"\n"}建议使用 CPU 推理方案。
          </p>
        </div>
      ) : (
        hw.gpus.map((g, i) => {
          const vendorColor = VENDOR_COLORS[g.vendor];
          const typeLabel = g.gpu_type === "Discrete"
            ? "独立显卡"
            : g.gpu_type === "Integrated"
              ? "集成显卡"
              : "其他适配器";
          return (
            <div
              key={i}
              className="card"
              style={{ borderLeft: `4px solid ${vendorColor}`, marginBottom: 12 }}
            >
              <div className="card-title">GPU[{i}] {typeLabel}</div>
              <div style={GPU_NAME_STYLE}>{g.name}</div>
              <div style={FLEX_WRAP_STYLE}>
                <span className="chip">厂商：{VENDOR_LABELS[g.vendor]}</span>
                <span className="chip">类型：{typeLabel}</span>
                <span className="chip">后端：{g.backend}</span>
                <span className="chip">
                  显存：{g.vram != null ? `${g.vram} MB (${(g.vram / 1024).toFixed(0)} GB)` : "未知"}
                </span>
              </div>
              <div style={GPU_META_STYLE}>
                设备 ID：{`0x${g.device_id[0].toString(16).padStart(4, "0")} / 0x${g.device_id[1].toString(16).padStart(4, "0")}`}
              </div>
            </div>
          );
        })
      )}

      <h3 style={SECTION_TITLE_STYLE}>本地大语言模型（{hw.local_models.length} 个）</h3>
      <p className="model-estimate-note">扫描 Ollama 与 LM Studio 的本地模型；结果仅按文件大小、显存和系统内存估算，不代表速度或最大上下文保证。</p>
      {hw.model_scan_warnings.map((warning) => <div className="warning-banner" role="status" key={warning}>{warning}</div>)}
      {hw.local_models.length === 0 ? (
        <div className="card"><span className="muted">未在 Ollama 或 LM Studio 模型目录中发现主模型文件。</span></div>
      ) : (
        <div className="model-grid">
          {hw.local_models.map((model) => (
            <div className="card model-card" key={`${model.source}-${model.path}`}>
              <div className="model-card-heading">
                <strong title={model.path}>{model.name}</strong>
                <span className={`model-fit fit-${model.fit}`}>{FIT_LABELS[model.fit]}</span>
              </div>
              <div className="model-meta">
                <span>{model.source === "LmStudio" ? "LM Studio" : "Ollama"}</span>
                <span>{(model.size_bytes / 1024 / 1024 / 1024).toFixed(1)} GB</span>
                {model.quantization && <span>{model.quantization}</span>}
              </div>
              <p>{model.fit_reason}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const HardwarePanel = memo(HardwarePanelInner);
HardwarePanel.displayName = "HardwarePanel";

export default HardwarePanel;
