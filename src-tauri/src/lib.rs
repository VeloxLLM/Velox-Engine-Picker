// Velox Engine Picker — Tauri Library
// 注册 Tauri 命令和插件

#[cfg(not(any(feature = "v1", feature = "v2", feature = "v3")))]
compile_error!("必须启用一个版本 feature：v1、v2 或 v3");

#[cfg(all(feature = "v1", not(feature = "v2"), not(target_os = "windows")))]
compile_error!("v1 是 Windows 专用稳定版；macOS 请使用 v2，Linux/ARM 请使用 v3");

#[cfg(all(
    feature = "v2",
    not(feature = "v3"),
    not(any(target_os = "windows", target_os = "macos"))
))]
compile_error!("v2 仅支持 Windows 和 macOS；Linux/ARM 请使用 v3");

mod commands;
mod engine;
mod hardware;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_hardware,
            commands::get_recommendation,
            commands::merge_recommendations_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
