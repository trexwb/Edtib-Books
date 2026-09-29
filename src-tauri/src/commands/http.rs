//! 统一 HTTP 命令：前端不再直连后端，所有网络请求统一在 Rust 侧发起。
//!
//! 说明：`src/bridge/channels.ts` 未声明 HTTP 相关通道（迁移前网络请求由前端 Axios 直接发起），
//! 本模块为迁移要求 3 提供 Rust 侧能力入口与统一出口，命令名以 `http_` 前缀，不与既有通道冲突。

use serde_json::Value;
use tauri::{AppHandle, State};

use crate::error::{ok_envelope, AppError, AppResult};
use crate::http;
use crate::paths;
use crate::state::AppState;

/// http_request：发起一次 HTTP 请求，返回 `{ status, ok, headers, contentType, data }`
#[tauri::command]
pub async fn http_request(
    state: State<'_, AppState>,
    url: String,
    method: Option<String>,
    headers: Option<Value>,
    body: Option<Value>,
) -> AppResult<Value> {
    let method = method.unwrap_or_else(|| "GET".to_string());
    let payload = state
        .http()
        .request(&method, &url, headers.as_ref(), body.as_ref())
        .await?;
    Ok(ok_envelope(payload.into_value()))
}

/// http_download：下载到本地文件，返回 `{ path, bytes }`。
/// `dir` 为空时落到应用缓存目录；`dir` 仅允许应用数据目录内的相对路径。
#[tauri::command]
pub async fn http_download(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    dir: Option<String>,
    file_name: Option<String>,
) -> AppResult<Value> {
    let base = match dir.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        Some(dir) => super::fs::resolve_input(&app, dir, false)?,
        None => paths::cache_dir(&app),
    };
    paths::ensure_dir(&base)?;

    let name = match file_name.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        Some(name) => name.to_string(),
        None => http::file_name_from_url(&url)
            .ok_or_else(|| AppError::bad_request(format!("无法从地址解析文件名：{url}")))?,
    };
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::bad_request(format!("非法文件名：{name}")));
    }

    let dest = base.join(name);
    let bytes = state.http().download_to(&url, &dest).await?;
    Ok(ok_envelope(serde_json::json!({
        "path": paths::to_string(&dest),
        "bytes": bytes,
    })))
}
