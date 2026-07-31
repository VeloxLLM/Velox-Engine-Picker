import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HardwareInfo, EngineRecommendation } from "./types";
import HardwarePanel from "./components/HardwarePanel";
import RecommendationPanel from "./components/RecommendationPanel";
import "./App.css";

type Tab = "recommendation" | "hardware";

function App() {
  const [tab, setTab] = useState<Tab>("recommendation");
  const [hw, setHw] = useState<HardwareInfo | null>(null);
  const [rec, setRec] = useState<EngineRecommendation | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const detect = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [hardware, recommendation] = await Promise.all([
        invoke<HardwareInfo>("detect_hardware"),
        invoke<EngineRecommendation>("get_recommendation"),
      ]);
      setHw(hardware);
      setRec(recommendation);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    detect();
  }, [detect]);

  return (
    <div className="app">
      {/* Top Bar */}
      <header className="top-bar">
        <h1 className="app-title">Velox Engine Picker</h1>
        <span className="version-badge">
          v1 &middot; Windows only
        </span>
        <div className="top-right">
          <button className="btn btn-primary" onClick={detect} disabled={loading}>
            {loading ? "检测中..." : "重新检测"}
          </button>
        </div>
      </header>

      {/* Sidebar + Content */}
      <div className="layout">
        <nav className="sidebar">
          <button
            className={`tab-btn ${tab === "recommendation" ? "active" : ""}`}
            onClick={() => setTab("recommendation")}
          >
            引擎推荐
          </button>
          <button
            className={`tab-btn ${tab === "hardware" ? "active" : ""}`}
            onClick={() => setTab("hardware")}
          >
            硬件信息
          </button>
          <div className="sidebar-footer">
            <small>VeloxLLM v0.1.0</small>
          </div>
        </nav>

        <main className="content">
          <div className="content-inner">
            {error && (
              <div className="error-banner">
                检测失败：{error}
              </div>
            )}

            {loading ? (
              <div className="loading">正在检测硬件信息...</div>
            ) : hw && rec ? (
              tab === "hardware" ? (
                <HardwarePanel hw={hw} />
              ) : (
                <RecommendationPanel hw={hw} rec={rec} />
              )
            ) : null}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;