// Velox Engine Picker — Tauri Entry Point
// 一键检测本机硬件配置，智能推荐 LLM 推理引擎

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::init();
    velox_engine_picker_lib::run()
}