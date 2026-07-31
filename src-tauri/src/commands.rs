//! Tauri IPC 命令
//!
//! 暴露给前端的 Rust 函数：
//! - detect_hardware() → HardwareInfo
//! - get_recommendation(hw) → EngineRecommendation
//! - merge_recommendations(recs) → EngineRecommendation

use crate::engine::{merge_recommendations, recommend};
use crate::engine::types::EngineRecommendation;
use crate::hardware::HardwareInfo;

/// 检测本机硬件信息（CPU / 内存 / GPU）
#[tauri::command]
pub fn detect_hardware() -> HardwareInfo {
    HardwareInfo::collect()
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
