//! 版本 / 更新 / 缓存命令（对应 `src/bridge/channels.ts` 的 `app.*`）。
//!
//! * `get_app_version`：与老实现同格式 `版本别名-vX.Y.Z(TIMER)`；
//! * `check_update` / `confirm_update` / `restart_app`：基于 `tauri-plugin-updater`，
//!   检查结果与下载进度通过 4 个更新事件下发（见 [`crate::update`] 与 `BRIDGE_EVENTS`）；
//! * `cache_file`：远端 URL 在 Rust 侧下载（前端不直连网络），本地文件复制进缓存目录。

use serde_json::{json, Value};
use tauri::{AppHandle, State};

use crate::crypt::sha256_hex;
use crate::error::{ok_envelope, AppError, AppResult};
use crate::http;
use crate::paths;
use crate::state::AppState;
use crate::update;

/// get_app_version：应用版本（老 `get-app-version`）
#[tauri::command]
pub async fn get_app_version() -> AppResult<String> {
    Ok(update::app_version())
}

/// check_update：检查更新并下发更新事件（老 `check-update`）
#[tauri::command]
pub async fn check_update(app: AppHandle) -> AppResult<Value> {
    let data = update::check_for_updates(&app).await?;
    Ok(ok_envelope(data))
}

/// confirm_update：确认安装已下载的更新（老 `confirm-update`）
#[tauri::command]
pub async fn confirm_update(app: AppHandle) -> AppResult<Value> {
    let data = update::confirm_update(&app)?;
    Ok(ok_envelope(data))
}

/// restart_app：重启应用（老 `restart-app`，该函数不返回）
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> AppResult<Value> {
    update::restart_app(&app)
}

/// cache_file：缓存文件到本地并返回绝对路径（老 `cache-file`）。
///
/// 入参为 URL（`app.cacheFile` 转发自图片地址）时在 Rust 侧下载；为本地路径时复制进缓存目录。
/// 返回值按 `src/bridge/types.ts` 的声明返回裸字符串路径（老实现返回信封，属于历史缺陷，
/// 会导致前端 `<img src>` 拿到对象；此处按前端声明的类型修正）。
#[tauri::command]
pub async fn cache_file(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> AppResult<String> {
    let input = url.trim();
    if input.is_empty() {
        return Err(AppError::bad_request("缓存地址不能为空"));
    }

    let cache_dir = paths::cache_dir(&app);
    paths::ensure_dir(&cache_dir)?;

    if http::is_remote_url(input) {
        let url = http::normalize_url(input)?;
        let dest = cache_dir.join(cached_name(&url));
        if dest.is_file() {
            // 同一 URL 命中同一缓存文件，避免重复下载
            return Ok(paths::to_string(&dest));
        }
        state.http().download_to(&url, &dest).await?;
        return Ok(paths::to_string(&dest));
    }

    // 本地文件：相对路径按应用数据目录解析，绝对路径需位于允许目录内
    let source = super::fs::resolve_input(&app, input, false)?;
    if !source.is_file() {
        return Err(AppError::not_found(format!(
            "本地文件不存在：{}",
            paths::to_string(&source)
        )));
    }
    let file_name = source
        .file_name()
        .map(|name| sanitize(name.to_string_lossy().as_ref()))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "cached.bin".to_string());
    let dest = cache_dir.join(format!("local-{file_name}"));
    std::fs::copy(&source, &dest).map_err(|err| {
        AppError::internal(format!(
            "缓存文件失败 {} -> {}：{err}",
            paths::to_string(&source),
            paths::to_string(&dest)
        ))
    })?;
    Ok(paths::to_string(&dest))
}

/// 缓存文件名：URL 摘要前缀保证不同 URL 不互相覆盖，保留原始文件名便于排查
fn cached_name(url: &str) -> String {
    let digest = sha256_hex(url);
    let short = digest.get(..16).unwrap_or(digest.as_str());
    match http::file_name_from_url(url) {
        Some(name) => {
            let name = sanitize(&name);
            if name.is_empty() {
                format!("{short}.bin")
            } else {
                format!("{short}-{name}")
            }
        }
        None => format!("{short}.bin"),
    }
}

/// 清洗文件名（去掉路径分隔与 `..`，仅保留常见安全字符）
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@'))
        .collect();
    let cleaned = cleaned.trim_start_matches('.').to_string();
    if cleaned.len() > 80 {
        cleaned.chars().take(80).collect()
    } else {
        cleaned
    }
}

/// bootstrap_report：启动自检报告（密钥来源 / 加密状态 / 表数量），便于排查与联调
#[tauri::command]
pub async fn bootstrap_report(state: State<'_, AppState>) -> AppResult<Value> {
    Ok(json!({
        "ready": state.ready(),
        "bootstrap": state.bootstrap_report(),
    }))
}
