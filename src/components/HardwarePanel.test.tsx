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
};

describe("HardwarePanel", () => {
  it("shows one dGPU detail and an explicit iGPU status", () => {
    render(<HardwarePanel hw={hw} />);
    expect(screen.getByText(/iGPU 检测：未检测到/)).toBeTruthy();
    expect(screen.getAllByText("NVIDIA GPU")).toHaveLength(1);
  });
});
