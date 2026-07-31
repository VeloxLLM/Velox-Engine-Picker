//! Discover local Ollama and LM Studio model files and estimate whether they fit.

use super::{GpuInfo, GpuType, MemoryInfo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalModelSource {
    Ollama,
    LmStudio,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFit {
    Gpu,
    Hybrid,
    Cpu,
    InsufficientMemory,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub name: String,
    pub source: LocalModelSource,
    pub path: String,
    pub size_bytes: u64,
    pub quantization: Option<String>,
    pub fit: ModelFit,
    pub fit_reason: String,
}

pub struct LocalModelDetectionResult {
    pub models: Vec<LocalModelInfo>,
    pub warnings: Vec<String>,
}

#[must_use]
pub fn collect_local_models(memory: &MemoryInfo, gpus: &[GpuInfo]) -> LocalModelDetectionResult {
    let mut models = Vec::new();
    let mut warnings = Vec::new();
    for root in ollama_roots() {
        scan_ollama(&root, &mut models, &mut warnings);
    }
    for root in lm_studio_roots() {
        scan_lm_studio(&root, &mut models, &mut warnings);
    }

    let mut seen = HashSet::new();
    models.retain(|model| seen.insert(model.path.to_ascii_lowercase()));
    for model in &mut models {
        let (fit, reason) = estimate_fit(model.size_bytes, memory, gpus);
        model.fit = fit;
        model.fit_reason = reason;
    }
    models.sort_by_key(|model| (fit_rank(&model.fit), model.size_bytes, model.name.clone()));
    LocalModelDetectionResult { models, warnings }
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn dedupe_existing(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| path.is_dir())
        .filter(|path| seen.insert(path.to_string_lossy().to_ascii_lowercase()))
        .collect()
}

fn ollama_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = std::env::var_os("OLLAMA_MODELS").filter(|value| !value.is_empty()) {
        roots.push(PathBuf::from(path));
    }
    if let Some(home) = user_home() {
        roots.push(home.join(".ollama").join("models"));
    }
    dedupe_existing(roots)
}

fn lm_studio_roots() -> Vec<PathBuf> {
    let Some(home) = user_home() else {
        return Vec::new();
    };
    let mut roots = Vec::new();
    let settings = home.join(".lmstudio").join("settings.json");
    if let Ok(text) = fs::read_to_string(settings) {
        if let Ok(value) = serde_json::from_str::<Value>(&text) {
            if let Some(folder) = value.get("downloadsFolder").and_then(Value::as_str) {
                roots.push(PathBuf::from(folder));
            }
        }
    }
    roots.push(home.join(".lmstudio").join("models"));
    dedupe_existing(roots)
}

fn scan_lm_studio(root: &Path, models: &mut Vec<LocalModelInfo>, warnings: &mut Vec<String>) {
    let mut files = Vec::new();
    if let Err(error) = walk_files(root, 0, 8, &mut files) {
        warnings.push(format!("LM Studio 模型目录读取失败: {error}"));
        return;
    }
    for path in files {
        let name = path
            .file_name()
            .and_then(|part| part.to_str())
            .unwrap_or_default();
        let lower = name.to_ascii_lowercase();
        if path
            .extension()
            .and_then(|part| part.to_str())
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("gguf"))
            || lower.starts_with("mmproj-")
            || lower.starts_with("mmproj_")
            || lower.contains(".mmproj")
        {
            continue;
        }
        if let Ok(metadata) = fs::metadata(&path) {
            models.push(LocalModelInfo {
                name: name.trim_end_matches(".gguf").to_string(),
                source: LocalModelSource::LmStudio,
                path: path.to_string_lossy().into_owned(),
                size_bytes: metadata.len(),
                quantization: infer_quantization(name),
                fit: ModelFit::Unknown,
                fit_reason: String::new(),
            });
        }
    }
}

fn scan_ollama(root: &Path, models: &mut Vec<LocalModelInfo>, warnings: &mut Vec<String>) {
    let manifests = root.join("manifests");
    if !manifests.is_dir() {
        return;
    }
    let mut files = Vec::new();
    if let Err(error) = walk_files(&manifests, 0, 8, &mut files) {
        warnings.push(format!("Ollama 模型清单读取失败: {error}"));
        return;
    }
    for manifest in files {
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(layer) = value
            .get("layers")
            .and_then(Value::as_array)
            .and_then(|layers| {
                layers.iter().find(|layer| {
                    layer
                        .get("mediaType")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| kind.contains("image.model"))
                })
            })
        else {
            continue;
        };
        let size = layer.get("size").and_then(Value::as_u64).unwrap_or(0);
        let digest = layer
            .get("digest")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let blob = root.join("blobs").join(digest.replace(':', "-"));
        let size_bytes = if size > 0 {
            size
        } else {
            fs::metadata(&blob).map(|m| m.len()).unwrap_or(0)
        };
        let name = ollama_model_name(&manifests, &manifest);
        models.push(LocalModelInfo {
            quantization: infer_quantization(&name),
            name,
            source: LocalModelSource::Ollama,
            path: manifest.to_string_lossy().into_owned(),
            size_bytes,
            fit: ModelFit::Unknown,
            fit_reason: String::new(),
        });
    }
}

fn walk_files(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    output: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    if depth > max_depth {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_files(&path, depth + 1, max_depth, output)?;
        } else if file_type.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn ollama_model_name(root: &Path, manifest: &Path) -> String {
    let parts: Vec<_> = manifest
        .strip_prefix(root)
        .unwrap_or(manifest)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.len() >= 2 {
        let tag = parts.last().cloned().unwrap_or_else(|| "latest".into());
        let model = parts.get(parts.len() - 2).cloned().unwrap_or_default();
        if parts.len() >= 3 && parts[parts.len() - 3] != "library" {
            return format!("{}/{model}:{tag}", parts[parts.len() - 3]);
        }
        return format!("{model}:{tag}");
    }
    manifest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未知模型")
        .to_string()
}

fn infer_quantization(name: &str) -> Option<String> {
    let upper = name.to_ascii_uppercase();
    const MARKERS: [&str; 11] = [
        "Q2_K", "Q3_K", "Q4_0", "Q4_1", "Q4_K_M", "Q4_K_S", "Q5_K_M", "Q5_K_S", "Q6_K", "Q8_0",
        "F16",
    ];
    MARKERS
        .iter()
        .find(|marker| upper.contains(**marker))
        .map(|marker| (*marker).to_string())
}

fn estimate_fit(size_bytes: u64, memory: &MemoryInfo, gpus: &[GpuInfo]) -> (ModelFit, String) {
    if size_bytes == 0 {
        return (ModelFit::Unknown, "模型大小未知，无法估算".into());
    }
    let weight_mb = size_bytes.div_ceil(1024 * 1024);
    let required_mb = weight_mb
        .saturating_mul(120)
        .div_ceil(100)
        .saturating_add(1024);
    let max_vram = gpus
        .iter()
        .filter(|gpu| gpu.gpu_type == GpuType::Discrete)
        .filter_map(|gpu| gpu.vram)
        .max();
    if let Some(vram) = max_vram {
        if required_mb <= vram.saturating_mul(9) / 10 {
            return (
                ModelFit::Gpu,
                format!(
                    "预计需约 {:.1} GB，适合完整载入显存",
                    required_mb as f64 / 1024.0
                ),
            );
        }
        if required_mb <= memory.total.saturating_mul(3) / 4 + vram.saturating_mul(4) / 5 {
            return (
                ModelFit::Hybrid,
                format!(
                    "显存不足以完整载入；预计需约 {:.1} GB，可尝试 GPU 分层卸载 + 系统内存",
                    required_mb as f64 / 1024.0
                ),
            );
        }
    }
    if required_mb <= memory.total.saturating_mul(3) / 4 {
        return (
            ModelFit::Cpu,
            format!(
                "预计需约 {:.1} GB，可尝试 CPU / 系统内存运行",
                required_mb as f64 / 1024.0
            ),
        );
    }
    (
        ModelFit::InsufficientMemory,
        format!(
            "预计需约 {:.1} GB，超过当前安全内存预算",
            required_mb as f64 / 1024.0
        ),
    )
}

fn fit_rank(fit: &ModelFit) -> u8 {
    match fit {
        ModelFit::Gpu => 0,
        ModelFit::Hybrid => 1,
        ModelFit::Cpu => 2,
        ModelFit::Unknown => 3,
        ModelFit::InsufficientMemory => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::{GpuBackend, GpuVendor};

    fn gpu(vram: u64) -> GpuInfo {
        GpuInfo {
            name: "Test GPU".into(),
            vendor: GpuVendor::Nvidia,
            gpu_type: GpuType::Discrete,
            vram: Some(vram),
            backend: GpuBackend::Dx12,
            device_id: (1, 1),
        }
    }

    #[test]
    fn small_model_fits_gpu() {
        assert_eq!(
            estimate_fit(
                3 * 1024 * 1024 * 1024,
                &MemoryInfo {
                    total: 32768,
                    available: 20000
                },
                &[gpu(8192)]
            )
            .0,
            ModelFit::Gpu
        );
    }

    #[test]
    fn large_model_uses_hybrid_memory() {
        assert_eq!(
            estimate_fit(
                17 * 1024 * 1024 * 1024,
                &MemoryInfo {
                    total: 32768,
                    available: 20000
                },
                &[gpu(8192)]
            )
            .0,
            ModelFit::Hybrid
        );
    }

    #[test]
    fn excludes_projection_marker_from_quantization_parsing() {
        assert_eq!(
            infer_quantization("Qwen3-8B-Q4_K_M.gguf").as_deref(),
            Some("Q4_K_M")
        );
    }
}
