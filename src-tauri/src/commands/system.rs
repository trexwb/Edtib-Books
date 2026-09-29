//! 系统信息命令（对应 `src/bridge/channels.ts` 的 `system.getInfo`）。
//!
//! 返回结构与 `src/bridge/index.ts#fallbackSystemInfo` 保持一致（前端在非 Tauri 环境下用同结构兜底）：
//! `{ platform, arch, hostname, cpus, totalMemory, freeMemory, versions, webview }`。
//!
//! 说明：Electron 版本号（node / chrome / electron）在 Tauri 下不再存在，统一返回空字符串，
//! 真实运行时版本通过 `webview`（WebView 内核版本）体现；`app` 版本由 `get_app_version` 提供。

use serde_json::{json, Value};

use crate::error::AppResult;

/// Tauri 侧平台名（对齐 Node `os.platform()` 取值：darwin / win32 / linux）
fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    }
}

/// 字节数格式化为 GB 字符串（老实现 `system-get-info` 的做法：(bytes / 1024^3).toFixed(2) + ' GB'）
fn format_gb(bytes: u64) -> String {
    format!("{:.2} GB", bytes as f64 / 1024f64.powi(3))
}

/// system_get_info：系统信息（老 `system-get-info`）
#[tauri::command]
pub async fn system_get_info() -> AppResult<Value> {
    use sysinfo::System;

    let system = System::new_all();
    let cpus = system.cpus().len();
    let total_memory = system.total_memory();
    let free_memory = system.available_memory();
    let hostname = System::host_name().unwrap_or_default();

    Ok(json!({
        "platform": platform_name(),
        "arch": std::env::consts::ARCH,
        "hostname": hostname,
        "cpus": cpus,
        "totalMemory": format_gb(total_memory),
        "freeMemory": format_gb(free_memory),
        // Tauri 下不存在 Node / Chromium / Electron 运行时，保持键结构一致但返回空串
        "versions": {
            "node": "",
            "chrome": "",
            "electron": "",
        },
        "webview": tauri::webview_version().unwrap_or_default(),
    }))
}
