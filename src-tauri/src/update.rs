//! 应用更新（替代老 Electron 的 electron-updater）。
//!
//! 前端契约（src/bridge/channels.ts / index.ts）：
//!   * 命令：`get_app_version`、`check_update`、`confirm_update`、`restart_app`；
//!   * 事件：`updater:update-available` / `updater:update-not-available` /
//!     `updater:download-progress` / `updater:update-downloaded`。
//!
//! 流程与老实现对齐（老实现用 electron-updater 的 checkForUpdatesAndNotify + autoDownload）：
//!   check_update  → 命中新版本则 emit update-available → 自动下载并 emit download-progress
//!                 → 下载完成 emit update-downloaded（同时把包体暂存，等待 confirm_update）
//!                 → 无新版本 / 出错 emit update-not-available（前端据此收起 loading 状态）
//!   confirm_update → 用 tauri-plugin-updater 安装已下载的包体（安装后由前端调用 restart_app 重启）
//!
//! ⚠️ 发布端需要提供 Tauri 更新清单（`latest.json`）与签名公钥，端点解析见 [`resolve_endpoints`]。

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::error::{AppError, AppResult};

/// 事件名（与前端 BRIDGE_EVENTS 严格一致）
pub const EVENT_UPDATE_AVAILABLE: &str = "updater:update-available";
pub const EVENT_UPDATE_NOT_AVAILABLE: &str = "updater:update-not-available";
pub const EVENT_DOWNLOAD_PROGRESS: &str = "updater:download-progress";
pub const EVENT_UPDATE_DOWNLOADED: &str = "updater:update-downloaded";

/// 构建期注入的版本别名 / 版本时间（对应老 main.ts 中的 packageJson.versionAlias / versionTimer）
const VERSION_ALIAS: &str = match option_env!("EDTIB_VERSION_ALIAS") {
    Some(value) => value,
    None => "unknown",
};
const VERSION_TIMER: &str = match option_env!("EDTIB_VERSION_TIMER") {
    Some(value) => value,
    None => "unknown",
};

/// 老实现的版本展示串：`${versionAlias}-v${version}(${versionTimer})`
pub fn app_version() -> String {
    format!("{}-v{}({})", VERSION_ALIAS, env!("CARGO_PKG_VERSION"), VERSION_TIMER)
}

/// 已下载待安装的更新包
struct Pending {
    update: Update,
    bytes: Vec<u8>,
}

/// 更新暂存状态（`check_update` 下载完成后填充，`confirm_update` 消费）
#[derive(Default)]
pub struct PendingUpdate {
    inner: Mutex<Option<Pending>>,
}

impl PendingUpdate {
    fn store(&self, update: Update, bytes: Vec<u8>) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(Pending { update, bytes });
        }
    }

    fn take(&self) -> AppResult<Option<Pending>> {
        self.inner
            .lock()
            .map(|mut guard| guard.take())
            .map_err(|_| AppError::internal("更新状态锁获取失败"))
    }
}

/// 更新信息（下发给前端的事件载荷）
#[derive(Debug, Clone)]
struct UpdateInfo {
    version: String,
    current_version: String,
    date: Option<String>,
    body: Option<String>,
    target: String,
}

impl UpdateInfo {
    fn from_update(update: &Update) -> Self {
        Self {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            date: update.date.map(|date| date.to_string()),
            body: update.body.clone(),
            target: update.target.clone(),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "currentVersion": self.current_version,
            "date": self.date,
            "releaseNotes": self.body,
            "target": self.target,
        })
    }
}

/// 更新服务端点解析（优先级：运行期环境变量 > 构建期环境变量 > tauri.conf.json plugins.updater.endpoints）
///
/// 说明：老 Electron 的 publish 配置为 generic provider
/// `https://static.edtib.com/update/books/${platform}/${arch}`（electron-updater 的 `latest.yml` 清单）。
/// Tauri 更新器需要**另一种**清单格式（`latest.json`，含 `platforms.<target>.url/signature`），
/// 因此端点需由发布端部署 Tauri 清单后再通过以下任一方式注入，代码不做臆测。
pub fn resolve_endpoints(app: &AppHandle) -> Vec<String> {
    let mut endpoints: Vec<String> = Vec::new();

    let mut push_all = |raw: &str| {
        for item in raw.split([',', ';', '\n']) {
            let item = item.trim();
            if !item.is_empty() && !endpoints.iter().any(|existing| existing == item) {
                endpoints.push(item.to_string());
            }
        }
    };

    if let Ok(value) = std::env::var("EDTIB_UPDATER_ENDPOINT") {
        push_all(&value);
    }
    if let Some(value) = option_env!("EDTIB_UPDATER_ENDPOINT") {
        push_all(value);
    }

    if let Some(Value::Object(updater)) = app.config().plugins.0.get("updater") {
        if let Some(Value::Array(list)) = updater.get("endpoints") {
            for item in list {
                if let Some(text) = item.as_str() {
                    push_all(text);
                }
            }
        }
    }

    endpoints
}

fn emit_error_event(app: &AppHandle, message: &str) {
    // 前端 UI 依赖 update-not-available 收起 loading（见 ClientsCheckUpdate.vue）
    let _ = app.emit(
        EVENT_UPDATE_NOT_AVAILABLE,
        json!({ "available": false, "message": message }),
    );
}

/// `check_update`：检查更新 → 下载 → 上报进度 → 暂存待安装包体
pub async fn check_for_updates(app: &AppHandle) -> AppResult<Value> {
    let endpoints = resolve_endpoints(app);
    if endpoints.is_empty() {
        let message = "更新服务地址未配置（EDTIB_UPDATER_ENDPOINT 或 tauri.conf.json plugins.updater.endpoints）";
        emit_error_event(app, message);
        return Err(AppError::unavailable(message));
    }

    let urls = endpoints
        .iter()
        .map(|endpoint| {
            tauri::Url::parse(endpoint)
                .map_err(|err| AppError::bad_request(format!("更新地址非法 {endpoint}：{err}")))
        })
        .collect::<AppResult<Vec<_>>>()?;

    let updater = app
        .updater_builder()
        .endpoints(urls)
        .map_err(updater_error)?
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(updater_error)?;

    let found = match updater.check().await {
        Ok(found) => found,
        Err(err) => {
            let message = format!("检查更新失败：{err}");
            emit_error_event(app, &message);
            return Err(AppError::internal(message));
        }
    };

    let Some(update) = found else {
        // 已是最新版本（含端点返回 204 的情况）
        let _ = app.emit(
            EVENT_UPDATE_NOT_AVAILABLE,
            json!({ "available": false, "currentVersion": app_version() }),
        );
        return Ok(json!({ "available": false, "currentVersion": app_version() }));
    };

    let info = UpdateInfo::from_update(&update);
    let _ = app.emit(EVENT_UPDATE_AVAILABLE, info.to_json());

    // 自动下载（对齐老实现的自动下载 + 进度上报）
    let last_emit = Mutex::new((Instant::now(), -1.0f64));
    let app_for_progress = app.clone();
    let downloaded = update
        .download(
            move |chunk: usize, total: Option<u64>| {
                let total = total.unwrap_or(0);
                let percent = if total > 0 {
                    (chunk as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                if let Ok(mut guard) = last_emit.lock() {
                    let (last_time, last_percent) = *guard;
                    let rounded = (percent * 10.0).round() / 10.0;
                    // 限流：进度变化 < 0.5% 且不足 200ms 时不上报，避免事件风暴
                    if rounded == last_percent
                        || (last_time.elapsed() < Duration::from_millis(200)
                            && (rounded - last_percent).abs() < 0.5)
                    {
                        return;
                    }
                    *guard = (Instant::now(), rounded);
                    let _ = app_for_progress.emit(
                        EVENT_DOWNLOAD_PROGRESS,
                        json!({
                            "percent": rounded,
                            "transferred": chunk,
                            "total": if total > 0 { Value::from(total) } else { Value::Null },
                        }),
                    );
                }
            },
            || {},
        )
        .await
        .map_err(|err| {
            let message = format!("更新包下载失败：{err}");
            emit_error_event(app, &message);
            AppError::internal(message)
        })?;

    let size = downloaded.len();
    app.state::<PendingUpdate>().store(update, downloaded);

    let mut payload = info.to_json();
    if let Some(object) = payload.as_object_mut() {
        object.insert("size".to_string(), Value::from(size as u64));
    }
    let _ = app.emit(EVENT_UPDATE_DOWNLOADED, payload.clone());

    Ok(json!({ "available": true, "update": info.to_json(), "size": size }))
}

/// `confirm_update`：安装已下载的更新包
pub fn confirm_update(app: &AppHandle) -> AppResult<Value> {
    let Some(pending) = app.state::<PendingUpdate>().take()? else {
        return Err(AppError::bad_request(
            "没有已下载的更新包，请先执行 check_update",
        ));
    };
    // 安装必须由**待安装的那个 `Update` 实例**执行（`Update::install`），
    // 而不是重新 build 出来的 `Updater`：全新 `Updater` 只有 check 能力，没有 install 方法。
    // `pending` 为 `take()` 得到的拥有所有权的值，此处对 `update` 借用、对 `bytes` 移动，
    // 二者是结构体的不同字段，借用与部分移动互不冲突。
    let Pending { update, bytes } = pending;
    let version = update.version.clone();
    update
        .install(bytes)
        .map_err(|err| AppError::internal(format!("更新安装失败：{err}")))?;
    Ok(json!({ "installed": true, "version": version }))
}

/// `restart_app`：重启当前应用（老实现 app.relaunch() + app.exit(0) 的等价物）
pub fn restart_app(app: &AppHandle) -> ! {
    app.restart()
}

fn updater_error(err: tauri_plugin_updater::Error) -> AppError {
    AppError::internal(format!("更新器初始化失败：{err}"))
}
