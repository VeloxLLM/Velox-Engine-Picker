import { invoke } from "@tauri-apps/api/core";
import { memo, useCallback, useEffect, useMemo, useState } from "react";
import type { HardwareInfo, ModelFit, OnlineModelCatalog, OnlineModelSource } from "../types";

interface Props { hw: HardwareInfo }
type SourceFilter = "All" | OnlineModelSource;

const FIT_LABELS: Record<ModelFit, string> = {
  Gpu: "适合独显",
  Hybrid: "GPU + 内存",
  Cpu: "CPU / 内存",
};
const QUICK_MODEL_FILTERS = ["Qwen", "Gemma", "DeepSeek", "Granite"] as const;

function OnlineModelsPanelInner({ hw }: Props) {
  const [catalog, setCatalog] = useState<OnlineModelCatalog | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [source, setSource] = useState<SourceFilter>("All");
  const [query, setQuery] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setCatalog(await invoke<OnlineModelCatalog>("get_online_models", { hw }));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setLoading(false);
    }
  }, [hw]);

  useEffect(() => { void refresh(); }, [refresh]);

  const visibleModels = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    return (catalog?.models ?? []).filter((model) =>
      (source === "All" || model.source === source)
      && (!needle || `${model.name} ${model.parameter_label}`.toLocaleLowerCase().includes(needle)),
    );
  }, [catalog, query, source]);

  const toggleQuickFilter = (modelName: string) => {
    setQuery((current) => current.trim().toLocaleLowerCase() === modelName.toLocaleLowerCase() ? "" : modelName);
  };

  return (
    <div>
      <div className="section-heading-row">
        <div>
          <h2>在线模型推荐</h2>
          <p>实时读取 Ollama 与 LM Studio 官方目录，只列出按本机内存预算可能运行的规格</p>
        </div>
        <button className="btn btn-primary" onClick={() => void refresh()} disabled={loading}>
          {loading ? "扫描中..." : "重新扫描"}
        </button>
      </div>

      <div className="model-toolbar" aria-label="在线模型筛选">
        <div className="model-source-tabs">
          {(["All", "Ollama", "LmStudio"] as const).map((value) => (
            <button key={value} className={source === value ? "active" : ""} onClick={() => setSource(value)}>
              {value === "All" ? "全部" : value === "LmStudio" ? "LM Studio" : value}
            </button>
          ))}
        </div>
        <input aria-label="搜索在线模型" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索模型名称或规格" />
      </div>

      <div className="model-estimate-note">按常见 Q4 量化估算，实际占用还受上下文长度、架构和运行后端影响。</div>
      {error && <div className="error-banner">在线目录扫描失败：{error}</div>}
      {catalog?.warnings.map((warning) => <div className="warning-banner" key={warning}>{warning}</div>)}

      {loading && !catalog ? (
        <div className="loading"><div className="loading-spinner" /><p>正在读取官方在线目录...</p></div>
      ) : (
        <>
          <div className="online-model-summary">
            <span className="online-model-count">共 {visibleModels.length} 个可选规格</span>
            <div className="quick-model-filters" aria-label="模型快捷筛选">
              {QUICK_MODEL_FILTERS.map((modelName) => (
                <button
                  type="button"
                  key={modelName}
                  className={query.trim().toLocaleLowerCase() === modelName.toLocaleLowerCase() ? "active" : ""}
                  aria-pressed={query.trim().toLocaleLowerCase() === modelName.toLocaleLowerCase()}
                  onClick={() => toggleQuickFilter(modelName)}
                >
                  {modelName}
                </button>
              ))}
            </div>
          </div>
          {visibleModels.length === 0 ? (
            <div className="card muted">没有找到符合当前筛选条件且可能运行的在线模型。</div>
          ) : (
            <div className="model-grid">
              {visibleModels.map((model) => (
                <article className="card model-card" key={`${model.source}-${model.name}-${model.parameter_label}`}>
                  <div className="model-card-heading">
                    <strong>{model.name}</strong>
                    <span className={`model-fit fit-${model.fit}`}>{FIT_LABELS[model.fit]}</span>
                  </div>
                  <div className="model-meta">
                    <span>{model.source === "LmStudio" ? "LM Studio" : "Ollama"}</span>
                    <span>{model.parameter_label}</span>
                    <span>Q4 约 {model.estimated_q4_gb.toFixed(1)} GB</span>
                  </div>
                  <p>{model.fit_reason}</p>
                  <div className="model-source-url" title={model.url}>{model.url}</div>
                </article>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

const OnlineModelsPanel = memo(OnlineModelsPanelInner);
OnlineModelsPanel.displayName = "OnlineModelsPanel";
export default OnlineModelsPanel;
