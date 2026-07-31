import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HardwareInfo, EngineRecommendation } from "../types";

export interface HardwareDetectionState {
  hw: HardwareInfo | null;
  rec: EngineRecommendation | null;
  loading: boolean;
  error: string | null;
  redetect: () => void;
}

function readableError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  try {
    return JSON.stringify(error);
  } catch {
    return "未知硬件检测错误";
  }
}

export function useHardwareDetection(): HardwareDetectionState {
  const [hw, setHw] = useState<HardwareInfo | null>(null);
  const [rec, setRec] = useState<EngineRecommendation | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const detect = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // 顺序调用：先检测硬件，再将结果传给推荐函数（避免双重检测）
      const hardware = await invoke<HardwareInfo>("detect_hardware");
      const recommendation = await invoke<EngineRecommendation>("get_recommendation", {
        hw: hardware,
      });
      setHw(hardware);
      setRec(recommendation);
    } catch (e) {
      setHw(null);
      setRec(null);
      setError(readableError(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    detect();
  }, [detect]);

  return { hw, rec, loading, error, redetect: detect };
}
