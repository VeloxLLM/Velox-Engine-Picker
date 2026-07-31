import { memo, useMemo } from "react";
import type { HardwareInfo, GpuVendor } from "../types";

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

  return (
    <div>
      <h2 style={HEADING_STYLE}>硬件配置</h2>
      <p style={SUBTITLE_STYLE}>以下信息由 sysinfo + wgpu 实时采集</p>

      {/* CPU */}
      <div className="card card-accent" style={{ borderLeftColor: "#3b82f6" }}>
        <div className="card-title">CPU</div>
        <div style={GPU_NAME_STYLE}>{hw.cpu.name}</div>
        <div style={FLEX_WRAP_STYLE}>
          <span className="chip">厂商：{hw.cpu.vendor}</span>
          <span className="chip">核心数：{hw.cpu.core_count}</span>
          <span className="chip">频率：{hw.cpu.frequency} MHz</span>
          <span className="chip">架构：{hw.cpu.arch}</span>
          <span className="chip">品牌：{BRAND_LABELS[hw.cpu.brand]}</span>
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
      <h3 style={SECTION_TITLE_STYLE}>显卡 / GPU（共 {hw.gpus.length} 个）</h3>

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
          const typeLabel = g.gpu_type === "Discrete" ? "独立显卡" : "集成显卡";
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
    </div>
  );
}

const HardwarePanel = memo(HardwarePanelInner);
HardwarePanel.displayName = "HardwarePanel";

export default HardwarePanel;
