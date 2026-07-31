import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { HardwareInfo, OnlineModelCatalog } from "../types";
import OnlineModelsPanel from "./OnlineModelsPanel";

const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const hw: HardwareInfo = {
  cpu: { name: "Test CPU", vendor: "Test", core_count: 8, logical_processor_count: 16, frequency: 3000, brand: "Other", arch: "X86_64", instruction_sets: [] },
  memory: { total: 32768, available: 16384 },
  gpus: [{ name: "NVIDIA GPU", vendor: "Nvidia", gpu_type: "Discrete", vram: 8192, backend: "Dx12", device_id: [0x10de, 1] }],
  platform: { os: "windows", arch: "x86_64", edition: "v1", support_level: "Stable" },
  availability: [],
  detection_warnings: [],
};

const catalog: OnlineModelCatalog = {
  models: [
    { name: "Qwen", source: "Ollama", parameter_label: "4B", estimated_q4_gb: 3.5, fit: "Gpu", fit_reason: "适合独显", url: "https://ollama.com/library/qwen" },
    { name: "Granite", source: "LmStudio", parameter_label: "3B", estimated_q4_gb: 2.9, fit: "Gpu", fit_reason: "适合独显", url: "https://lmstudio.ai/models/granite" },
  ],
  warnings: [],
};

describe("OnlineModelsPanel", () => {
  beforeEach(() => invokeMock.mockReset().mockResolvedValue(catalog));

  it("loads both official catalogs and filters by source", async () => {
    render(<OnlineModelsPanel hw={hw} />);

    await waitFor(() => expect(screen.getByText("Qwen")).toBeTruthy());
    expect(screen.getByText("Granite")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("get_online_models", { hw });

    fireEvent.click(screen.getByRole("button", { name: "LM Studio" }));
    expect(screen.queryByText("Qwen")).toBeNull();
    expect(screen.getByText("Granite")).toBeTruthy();
  });
});
