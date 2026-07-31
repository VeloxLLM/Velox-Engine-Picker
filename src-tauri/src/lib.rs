// Velox Engine Picker — Tauri Library
// 注册 Tauri 命令和插件

mod commands;
mod hardware;
mod engine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_hardware,
            commands::get_recommendation,
            commands::merge_recommendations_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}