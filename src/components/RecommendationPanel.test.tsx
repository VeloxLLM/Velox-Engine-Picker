import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import RecommendationPanel from "./RecommendationPanel";
import type { EngineRecommendation, HardwareInfo } from "../types";

const hw: HardwareInfo = {
  cpu: {
    name: "Test CPU",
    vendor: "Test",
    core_count: 4,
    logical_processor_count: 8,
    frequency: 3000,
    brand: "Other",
    arch: "X86_64",
    instruction_sets: ["AVX2"],
  },
  memory: { total: 16384, available: 8192 },
  gpus: [],
  platform: { os: "windows", arch: "x86_64", edition: "v1", support_level: "Stable" },
  detection_warnings: [],
  availability: [{
    target: { engine: "OpenVino", backend: "Cpu" },
    status: "CompatibleMissingRuntime",
    version: null,
    evidence: "未检测到 OpenVINO Runtime",
  }],
};

const recommendation: EngineRecommendation = {
  primary: { engine: "OpenVino", backend: "Cpu" },
  theoretical_primary: { engine: "OpenVino", backend: "Cpu" },
  ready_primary: null,
  alternatives: [{ engine: "LlamaCpp", backend: "Cpu" }],
  reasons: ["CPU-only test"],
  warnings: ["未检测到可直接运行的本地推理 Runtime"],
  memory_tip: null,
  session_id: "test",
  session_ts: 0,
};

describe("RecommendationPanel", () => {
  it("separates theoretical advice from locally runnable engines", () => {
    render(<RecommendationPanel hw={hw} rec={recommendation} />);
    expect(screen.getByText("理论最佳方案（硬件兼容性）")).toBeTruthy();
    expect(screen.getByText("未检测到可直接运行的推理环境")).toBeTruthy();
    expect(screen.queryByText("本机运行时可用性")).toBeNull();
    expect(screen.queryByText("兼容，缺少运行时")).toBeNull();
  });
});
