import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import HardwarePanel from "./HardwarePanel";
import type { HardwareInfo } from "../types";

const hw: HardwareInfo = {
  cpu: { name: "Test CPU", vendor: "Test", core_count: 8, logical_processor_count: 16, frequency: 3000, brand: "Other", arch: "X86_64", instruction_sets: [] },
  memory: { total: 32768, available: 16384 },
  gpus: [{ name: "NVIDIA GPU", vendor: "Nvidia", gpu_type: "Discrete", vram: 8192, backend: "Dx12", device_id: [0x10de, 1] }],
  platform: { os: "windows", arch: "x86_64", edition: "v1", support_level: "Stable" },
  availability: [],
  detection_warnings: [],
  local_models: [{ name: "Qwen-Test-Q4_K_M", source: "LmStudio", path: "C:\\models\\qwen.gguf", size_bytes: 4 * 1024 ** 3, quantization: "Q4_K_M", fit: "Gpu", fit_reason: "适合完整载入显存" }],
  model_scan_warnings: [],
};

describe("HardwarePanel", () => {
  it("shows explicit iGPU status and discovered local models", () => {
    render(<HardwarePanel hw={hw} />);
    expect(screen.getByText(/未检测到 iGPU/)).toBeTruthy();
    expect(screen.getByText("Qwen-Test-Q4_K_M")).toBeTruthy();
    expect(screen.getByText("适合 GPU")).toBeTruthy();
  });
});
