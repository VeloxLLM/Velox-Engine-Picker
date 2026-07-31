//! Tauri IPC 命令
//!
//! 暴露给前端的 Rust 函数：
//! - detect_hardware() → HardwareInfo
//! - get_recommendation(hw) → EngineRecommendation
//! - merge_recommendations(recs) → EngineRecommendation

use crate::engine::types::EngineRecommendation;
use crate::engine::{merge_recommendations, recommend};
use crate::hardware::HardwareInfo;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DetectionError {
    pub code: &'static str,
    pub message: String,
}

trait HardwareProbe {
    fn detect(&self) -> Result<HardwareInfo, DetectionError>;
}

struct SystemHardwareProbe;

impl HardwareProbe for SystemHardwareProbe {
    fn detect(&self) -> Result<HardwareInfo, DetectionError> {
        let hardware = HardwareInfo::collect();
        if let Some(message) = hardware.detection_warnings.first() {
            return Err(DetectionError {
                code: "GPU_DETECTION_FAILED",
                message: message.clone(),
            });
        }
        Ok(hardware)
    }
}

fn detect_with_probe(probe: &dyn HardwareProbe) -> Result<HardwareInfo, DetectionError> {
    probe.detect()
}

/// 检测本机硬件信息（CPU / 内存 / GPU）
#[tauri::command]
pub async fn detect_hardware() -> Result<HardwareInfo, DetectionError> {
    tauri::async_runtime::spawn_blocking(|| detect_with_probe(&SystemHardwareProbe))
        .await
        .map_err(|error| DetectionError {
            code: "DETECTION_TASK_FAILED",
            message: error.to_string(),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailedProbe;

    impl HardwareProbe for FailedProbe {
        fn detect(&self) -> Result<HardwareInfo, DetectionError> {
            Err(DetectionError {
                code: "GPU_DETECTION_FAILED",
                message: "DXGI enumeration failed".to_string(),
            })
        }
    }

    #[test]
    fn preserves_structured_probe_failure() {
        assert_eq!(
            detect_with_probe(&FailedProbe).unwrap_err(),
            DetectionError {
                code: "GPU_DETECTION_FAILED",
                message: "DXGI enumeration failed".to_string(),
            }
        );
    }
}

/// 基于硬件信息获取引擎推荐结果（接受前端传入的 HardwareInfo，避免重复检测）
#[tauri::command]
pub fn get_recommendation(hw: HardwareInfo) -> EngineRecommendation {
    recommend(&hw)
}

/// 合并多次检测的推荐结果
#[tauri::command]
pub fn merge_recommendations_cmd(
    recs: Vec<EngineRecommendation>,
) -> Result<EngineRecommendation, String> {
    merge_recommendations(&recs).ok_or_else(|| "No recommendations to merge".to_string())
}
