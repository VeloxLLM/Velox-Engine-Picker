import { useState } from "react";
import { useHardwareDetection } from "./hooks/useHardwareDetection";
import HardwarePanel from "./components/HardwarePanel";
import RecommendationPanel from "./components/RecommendationPanel";
import "./App.css";

type Tab = "recommendation" | "hardware";

function App() {
  const [tab, setTab] = useState<Tab>("recommendation");
  const { hw, rec, loading, error, redetect } = useHardwareDetection();

  return (
    <div className="app">
      {/* Top Bar */}
      <header className="top-bar">
        <h1 className="app-title">Velox Engine Picker</h1>
        <span className="version-badge">
          v1 &middot; Windows only
        </span>
        <div className="top-right">
          <button className="btn btn-primary" onClick={redetect} disabled={loading}>
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
            {loading ? (
              <div className="loading">
                <div className="loading-spinner" />
                <p>正在检测硬件信息...</p>
              </div>
            ) : error ? (
              <div className="empty-state">
                <div className="empty-icon">⚠️</div>
                <h3>检测失败</h3>
                <p className="error-detail">{error}</p>
                <button className="btn btn-primary" onClick={redetect}>
                  重新尝试
                </button>
              </div>
            ) : hw && rec ? (
              tab === "hardware" ? (
                <HardwarePanel hw={hw} />
              ) : (
                <RecommendationPanel hw={hw} rec={rec} />
              )
            ) : (
              <div className="empty-state">
                <div className="empty-icon">🔍</div>
                <h3>准备就绪</h3>
                <p>点击按钮开始检测本机硬件配置并获取引擎推荐</p>
                <button className="btn btn-primary" onClick={redetect}>
                  开始检测
                </button>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
