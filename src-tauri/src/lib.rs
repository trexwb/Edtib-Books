#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 骨架自检用示例命令：用于验证前端 -> Rust 的 IPC 通道可用。
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{name}！来自 Rust 侧的问候。")
}
