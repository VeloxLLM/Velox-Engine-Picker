import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useHardwareDetection } from "./useHardwareDetection";

const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const hardware = {
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
  availability: [],
  detection_warnings: [],
};

const recommendation = {
  primary: { engine: "OpenVino", backend: "Cpu" },
  theoretical_primary: { engine: "OpenVino", backend: "Cpu" },
  ready_primary: null,
  alternatives: [],
  reasons: [],
  warnings: [],
  memory_tip: null,
  session_id: "test",
  session_ts: 0,
};

describe("useHardwareDetection", () => {
  beforeEach(() => invokeMock.mockReset());

  it("loads hardware and its recommendation in order", async () => {
    invokeMock.mockImplementation((command: string) =>
      Promise.resolve(command === "detect_hardware" ? hardware : recommendation),
    );
    const { result } = renderHook(() => useHardwareDetection());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.hw).toEqual(hardware);
    expect(result.current.rec).toEqual(recommendation);
    expect(invokeMock.mock.calls.map((call) => call[0])).toEqual([
      "detect_hardware",
      "get_recommendation",
    ]);
  });

  it("surfaces a structured detection error and supports retry", async () => {
    invokeMock.mockRejectedValueOnce({ code: "DETECTION_TASK_FAILED", message: "GPU probe failed" });
    const { result } = renderHook(() => useHardwareDetection());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe("GPU probe failed");

    invokeMock.mockImplementation((command: string) =>
      Promise.resolve(command === "detect_hardware" ? hardware : recommendation),
    );
    await act(async () => result.current.redetect());
    expect(result.current.error).toBeNull();
    expect(result.current.hw).toEqual(hardware);
  });
});
