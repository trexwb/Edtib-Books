//! edtib books（紧固助手 · 个人版）Tauri 后端入口。
//!
//! 迁移要点（对应 `edtib-books-前端迁移报告.md` 的遗留事项）：
//!   1. 前端 `src/bridge/channels.ts` 声明的命令与更新事件在 [`commands`] 中实现；
//!   2. 本地数据库固定使用 SQLCipher 加密，密钥来自系统 Keychain（每设备随机生成并持久化，
//!      首次启动完成初始化），见 [`keychain`] 与 [`db::connection`]；
//!   3. 所有 HTTP 请求统一在 Rust 侧发起（[`http`]），前端不直连后端；
//!   4. 应用更新使用 `tauri-plugin-updater`（[`update`]）；
//!   5. 不迁移老 Electron 侧的 `VITE_APP_SECRET` / `app_secret` 密钥链，新项目不保留该逻辑
//!      （[`crypt`] 仅在显式配置环境变量时启用字段加密，未配置即透传）；
//!   6. 不修改 `tauri.conf.json` 已有配置（identifier / frontendDist 等保持原样）。

mod commands;
mod crypt;
mod db;
mod error;
mod http;
mod keychain;
mod paths;
mod state;
mod update;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::crypt::FieldCrypt;
use crate::db::{connection, Database};
use crate::error::AppResult;

/// 启动应用
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = state::AppState::new();
            match bootstrap(&handle, &state) {
                Ok(report) => {
                    println!("[bootstrap] 数据层就绪：{report}");
                }
                Err(error) => {
                    // 不阻断应用启动：命令层会以「数据层不可用」的错误回显，前端按既有逻辑降级 HTTP
                    eprintln!("[bootstrap] 数据层初始化失败：{error}");
                    state.set_startup_error(error.to_string());
                }
            }
            app.manage(state);
            // 更新暂存状态：check_update 下载完成后写入，confirm_update 消费
            app.manage(update::PendingUpdate::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // db.*
            commands::db::db_find_all,
            commands::db::db_get_list,
            commands::db::db_find_one,
            commands::db::db_create,
            commands::db::db_update,
            commands::db::db_delete,
            commands::db::db_bulk_create,
            // fs.*
            commands::fs::fs_read_file,
            commands::fs::fs_write_file,
            commands::fs::fs_delete_file,
            commands::fs::fs_list_files,
            commands::fs::fs_get_user_data_path,
            commands::fs::fs_get_documents_path,
            commands::fs::fs_exists,
            // system.*
            commands::system::system_get_info,
            // app.*
            commands::app::get_app_version,
            commands::app::check_update,
            commands::app::confirm_update,
            commands::app::restart_app,
            commands::app::cache_file,
            // 统一 HTTP（Rust 侧发起）
            commands::http::http_request,
            commands::http::http_download,
            // 启动自检
            commands::app::bootstrap_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 启动阶段初始化数据层：
///   1. 解析应用数据目录；
///   2. 从系统 Keychain 读取 SQLCipher 密钥，不存在则随机生成并持久化（首次启动初始化）；
///   3. 首次启动且存在老 Electron 明文库时，通过 `sqlcipher_export` 搬迁为加密库；
///   4. 打开加密库并执行幂等迁移（建表）；
///   5. 写入共享状态并生成自检报告。
fn bootstrap(app: &AppHandle, state: &state::AppState) -> AppResult<Value> {
    let data_dir = paths::data_dir(app);
    paths::ensure_dir(&data_dir)?;

    // 1) SQLCipher 密钥（Keychain 优先，每设备随机生成后持久化）
    let key = keychain::load_or_create(&data_dir)?;

    // 2) 首次启动：老 Electron 明文库搬迁（仅当新库不存在时执行）
    let db_path = paths::db_file(app);
    let first_launch = !db_path.exists();
    let legacy_import = if first_launch {
        let legacy = paths::legacy_db_file();
        if legacy.is_file() {
            match connection::import_plain_sqlite(&db_path, &legacy, &key.hex) {
                Ok(()) => json!({ "imported": true, "source": paths::to_string(&legacy) }),
                Err(error) => {
                    // 搬迁失败则回退为全新加密库，避免留下半成品文件
                    let _ = std::fs::remove_file(&db_path);
                    json!({
                        "imported": false,
                        "source": paths::to_string(&legacy),
                        "error": error.to_string(),
                    })
                }
            }
        } else {
            json!({ "imported": false, "source": Value::Null, "reason": "未发现旧版数据库" })
        }
    } else {
        json!({ "imported": false, "reason": "非首次启动，跳过旧库搬迁" })
    };

    // 3) 打开加密库并执行迁移
    let database = Database::open(&db_path, &key)?;
    let missing_tables = database.migrate()?;
    let tables = database.table_names()?;

    // 4) 字段加解密器（未配置密钥时透传，不内置任何默认密钥）
    let crypt = FieldCrypt::from_env();

    let report = json!({
        "ready": true,
        "dataDir": paths::to_string(&data_dir),
        "dbFile": paths::to_string(database.path()),
        "encrypted": database.encrypted(),
        "cipherVersion": database.cipher_version(),
        "keySource": database.key_source(),
        "tableCount": tables.len(),
        "missingTables": missing_tables,
        "firstLaunch": first_launch,
        "legacyImport": legacy_import,
        "fieldCryptEnabled": crypt.enabled(),
    });

    state.init_data_layer(database, crypt, report.clone())?;
    Ok(report)
}
