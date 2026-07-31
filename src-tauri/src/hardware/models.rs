//! Fetch public Ollama and LM Studio catalogs and estimate which models fit this machine.

use super::{GpuInfo, GpuType, HardwareInfo, MemoryInfo};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

const OLLAMA_LIBRARY_URL: &str = "https://ollama.com/library";
const LM_STUDIO_CATALOG_URL: &str = "https://lmstudio.ai/models";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnlineModelSource {
    Ollama,
    LmStudio,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFit {
    Gpu,
    Hybrid,
    Cpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineModelInfo {
    pub name: String,
    pub source: OnlineModelSource,
    pub parameter_label: String,
    pub estimated_q4_gb: f64,
    pub fit: ModelFit,
    pub fit_reason: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineModelCatalog {
    pub models: Vec<OnlineModelInfo>,
    pub warnings: Vec<String>,
}

#[must_use]
pub async fn collect_online_models(hw: &HardwareInfo) -> OnlineModelCatalog {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Velox-Engine-Picker/0.1")
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return OnlineModelCatalog {
                models: Vec::new(),
                warnings: vec![format!("在线模型客户端初始化失败: {error}")],
            };
        }
    };

    let mut models = Vec::new();
    let mut warnings = Vec::new();
    match fetch_text(&client, OLLAMA_LIBRARY_URL).await {
        Ok(html) => models.extend(parse_ollama_catalog(&html, &hw.memory, &hw.gpus)),
        Err(error) => warnings.push(format!("Ollama 在线目录读取失败: {error}")),
    }
    match fetch_text(&client, LM_STUDIO_CATALOG_URL).await {
        Ok(html) => models.extend(parse_lm_studio_catalog(&html, &hw.memory, &hw.gpus)),
        Err(error) => warnings.push(format!("LM Studio 在线目录读取失败: {error}")),
    }

    finish_catalog(models, warnings)
}

fn finish_catalog(mut models: Vec<OnlineModelInfo>, warnings: Vec<String>) -> OnlineModelCatalog {
    let mut seen = HashSet::new();
    models.retain(|model| {
        seen.insert((
            model.source.clone(),
            model.name.to_ascii_lowercase(),
            model.parameter_label.to_ascii_lowercase(),
        ))
    });
    models.sort_by(|a, b| {
        fit_rank(&a.fit)
            .cmp(&fit_rank(&b.fit))
            .then_with(|| a.estimated_q4_gb.total_cmp(&b.estimated_q4_gb))
            .then_with(|| a.name.cmp(&b.name))
    });
    OnlineModelCatalog { models, warnings }
}

async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.text().await.map_err(|error| error.to_string())
}

fn parse_ollama_catalog(html: &str, memory: &MemoryInfo, gpus: &[GpuInfo]) -> Vec<OnlineModelInfo> {
    let document = Html::parse_document(html);
    let card_selector = Selector::parse("#repo li a[href^='/library/']").expect("valid selector");
    let title_selector = Selector::parse("[title]").expect("valid selector");
    let badge_selector = Selector::parse("span").expect("valid selector");
    let mut models = Vec::new();

    for card in document.select(&card_selector).take(40) {
        let href = card.value().attr("href").unwrap_or_default();
        let name = card
            .select(&title_selector)
            .find_map(|element| element.value().attr("title"))
            .unwrap_or_else(|| href.trim_start_matches("/library/"));
        if name.is_empty() {
            continue;
        }
        let labels = card
            .select(&badge_selector)
            .filter_map(|element| parse_parameter_label(&element.text().collect::<String>()))
            .map(|(label, value)| (label, OrderedFloat(value)))
            .collect::<HashSet<_>>();
        for (label, parameters_b) in labels {
            push_if_runnable(
                &mut models,
                name,
                OnlineModelSource::Ollama,
                &label,
                parameters_b.0,
                format!("https://ollama.com{href}"),
                memory,
                gpus,
            );
        }
    }
    models
}

fn parse_lm_studio_catalog(
    html: &str,
    memory: &MemoryInfo,
    gpus: &[GpuInfo],
) -> Vec<OnlineModelInfo> {
    let document = Html::parse_document(html);
    let card_selector = Selector::parse("a[href^='/models/']").expect("valid selector");
    let heading_selector =
        Selector::parse("h1, h2, h3, h4, .text-lg.font-medium").expect("valid selector");
    let mut models = Vec::new();

    for card in document.select(&card_selector).take(40) {
        let href = card.value().attr("href").unwrap_or_default();
        let name = card
            .select(&heading_selector)
            .next()
            .map(|heading| clean_text(&heading.text().collect::<Vec<_>>().join(" ")))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| model_name_from_href(href));
        if name.is_empty() {
            continue;
        }
        let text = clean_text(&card.text().collect::<Vec<_>>().join(" "));
        for (label, parameters_b) in extract_parameter_labels(&text) {
            push_if_runnable(
                &mut models,
                &name,
                OnlineModelSource::LmStudio,
                &label,
                parameters_b.0,
                format!("https://lmstudio.ai{href}"),
                memory,
                gpus,
            );
        }
    }
    models
}

fn model_name_from_href(href: &str) -> String {
    href.trim_start_matches("/models/")
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(clippy::too_many_arguments)]
fn push_if_runnable(
    models: &mut Vec<OnlineModelInfo>,
    name: &str,
    source: OnlineModelSource,
    parameter_label: &str,
    parameters_b: f64,
    url: String,
    memory: &MemoryInfo,
    gpus: &[GpuInfo],
) {
    let estimated_q4_gb = parameters_b * 0.58 + 1.2;
    if let Some((fit, fit_reason)) = estimate_fit(estimated_q4_gb, memory, gpus) {
        models.push(OnlineModelInfo {
            name: name.to_string(),
            source,
            parameter_label: parameter_label.to_string(),
            estimated_q4_gb: (estimated_q4_gb * 10.0).round() / 10.0,
            fit,
            fit_reason,
            url,
        });
    }
}

fn estimate_fit(
    estimated_gb: f64,
    memory: &MemoryInfo,
    gpus: &[GpuInfo],
) -> Option<(ModelFit, String)> {
    let max_vram_gb = gpus
        .iter()
        .filter(|gpu| gpu.gpu_type == GpuType::Discrete)
        .filter_map(|gpu| gpu.vram)
        .max()
        .map(|mb| mb as f64 / 1024.0);
    let memory_gb = memory.total as f64 / 1024.0;
    if max_vram_gb.is_some_and(|vram| estimated_gb <= vram * 0.9) {
        return Some((
            ModelFit::Gpu,
            format!("Q4 预计约 {estimated_gb:.1} GB，可完整载入独显"),
        ));
    }
    if let Some(vram) = max_vram_gb {
        if estimated_gb <= memory_gb * 0.7 + vram * 0.8 {
            return Some((
                ModelFit::Hybrid,
                format!("Q4 预计约 {estimated_gb:.1} GB，可尝试 GPU 分层卸载"),
            ));
        }
    }
    (estimated_gb <= memory_gb * 0.7).then(|| {
        (
            ModelFit::Cpu,
            format!("Q4 预计约 {estimated_gb:.1} GB，可尝试 CPU / 内存运行"),
        )
    })
}

fn parse_parameter_label(text: &str) -> Option<(String, f64)> {
    let normalized = text.trim().to_ascii_lowercase();
    let regex = Regex::new(r"^(?:(\d+(?:\.\d+)?)x)?(\d+(?:\.\d+)?)([bm])$").expect("valid regex");
    let captures = regex.captures(&normalized)?;
    let multiplier = captures
        .get(1)
        .map_or(1.0, |value| value.as_str().parse().unwrap_or(1.0));
    let value: f64 = captures.get(2)?.as_str().parse().ok()?;
    let unit = captures.get(3)?.as_str();
    let parameters_b = multiplier * value * if unit == "m" { 0.001 } else { 1.0 };
    (parameters_b >= 0.1).then(|| (normalized.to_ascii_uppercase(), parameters_b))
}

fn extract_parameter_labels(text: &str) -> HashSet<(String, OrderedFloat)> {
    let regex = Regex::new(r"(?i)(?:\d+(?:\.\d+)?x)?\d+(?:\.\d+)?[bm]\b").expect("valid regex");
    regex
        .find_iter(text)
        .filter_map(|matched| parse_parameter_label(matched.as_str()))
        .map(|(label, value)| (label, OrderedFloat(value)))
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct OrderedFloat(f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for OrderedFloat {}
impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fit_rank(fit: &ModelFit) -> u8 {
    match fit {
        ModelFit::Gpu => 0,
        ModelFit::Hybrid => 1,
        ModelFit::Cpu => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::{GpuBackend, GpuVendor};

    fn hardware() -> (MemoryInfo, Vec<GpuInfo>) {
        (
            MemoryInfo {
                total: 32768,
                available: 16000,
            },
            vec![GpuInfo {
                name: "RTX".into(),
                vendor: GpuVendor::Nvidia,
                gpu_type: GpuType::Discrete,
                vram: Some(8192),
                backend: GpuBackend::Dx12,
                device_id: (1, 1),
            }],
        )
    }

    #[test]
    fn parses_ollama_cards_and_filters_oversized_variants() {
        let (memory, gpus) = hardware();
        let html = r#"<div id='repo'><li><a href='/library/qwen'><div title='qwen'></div><span>4b</span><span>72b</span></a></li></div>"#;
        let models = parse_ollama_catalog(html, &memory, &gpus);
        assert!(models.iter().any(|model| model.parameter_label == "4B"));
        assert!(!models.iter().any(|model| model.parameter_label == "72B"));
    }

    #[test]
    fn parses_moe_parameter_label() {
        assert_eq!(parse_parameter_label("8x7b").map(|item| item.1), Some(56.0));
    }

    #[test]
    fn parses_lm_studio_cards() {
        let (memory, gpus) = hardware();
        let html = r#"<a href='/models/granite'><h3>Granite 4.1</h3><span>3B</span><span>8B</span><p>16.9K downloads</p></a>"#;
        let models = parse_lm_studio_catalog(html, &memory, &gpus);
        assert!(models
            .iter()
            .any(|model| model.name == "Granite 4.1" && model.parameter_label == "3B"));
        assert!(models.iter().any(|model| model.parameter_label == "8B"));
    }

    #[test]
    fn combined_catalog_does_not_drop_the_second_source() {
        let mut models = (0..80)
            .map(|index| OnlineModelInfo {
                name: format!("Ollama {index}"),
                source: OnlineModelSource::Ollama,
                parameter_label: "1B".into(),
                estimated_q4_gb: 1.8,
                fit: ModelFit::Gpu,
                fit_reason: "test".into(),
                url: format!("https://ollama.com/library/model-{index}"),
            })
            .collect::<Vec<_>>();
        models.push(OnlineModelInfo {
            name: "Granite".into(),
            source: OnlineModelSource::LmStudio,
            parameter_label: "3B".into(),
            estimated_q4_gb: 2.9,
            fit: ModelFit::Gpu,
            fit_reason: "test".into(),
            url: "https://lmstudio.ai/models/granite".into(),
        });

        let catalog = finish_catalog(models, Vec::new());

        assert_eq!(catalog.models.len(), 81);
        assert!(catalog
            .models
            .iter()
            .any(|model| model.source == OnlineModelSource::LmStudio));
    }

    #[test]
    #[ignore = "requires access to official online catalogs"]
    fn live_official_catalogs_return_runnable_models() {
        let (memory, gpus) = hardware();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("Velox-Engine-Picker/catalog-smoke-test")
            .build()
            .expect("client");
        let (ollama_html, lm_studio_html) = tauri::async_runtime::block_on(async {
            (
                fetch_text(&client, OLLAMA_LIBRARY_URL)
                    .await
                    .expect("Ollama catalog"),
                fetch_text(&client, LM_STUDIO_CATALOG_URL)
                    .await
                    .expect("LM Studio catalog"),
            )
        });
        let ollama = parse_ollama_catalog(&ollama_html, &memory, &gpus);
        let lm_studio = parse_lm_studio_catalog(&lm_studio_html, &memory, &gpus);
        eprintln!(
            "Ollama runnable variants: {}, LM Studio runnable variants: {}",
            ollama.len(),
            lm_studio.len()
        );
        assert!(!ollama.is_empty());
        assert!(!lm_studio.is_empty());
    }
}
