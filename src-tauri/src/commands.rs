//! Tauri IPC 命令
//!
//! 暴露给前端的 Rust 函数：
//! - detect_hardware() → HardwareInfo
//! - get_recommendation() → EngineRecommendation

use crate::engine::recommend;
use crate::engine::types::EngineRecommendation;
use crate::hardware::HardwareInfo;

/// 检测本机硬件信息（CPU / 内存 / GPU）
#[tauri::command]
pub fn detect_hardware() -> HardwareInfo {
    HardwareInfo::collect()
}

/// 基于硬件信息获取引擎推荐结果
#[tauri::command]
pub fn get_recommendation() -> EngineRecommendation {
    let hw = HardwareInfo::collect();
    recommend(&hw)
}