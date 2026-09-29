//! 应用运行期共享状态。
//!
//! 由 `tauri::Builder::manage` 在 setup 阶段注入，命令层通过 `State<'_, AppState>` 取用：
//!   * `database`：SQLCipher 加密数据库（启动阶段初始化，失败时给出可读错误）；
//!   * `crypt`：字段级加解密器（读环境变量，未配置时透传）；
//!   * `http`：统一 HTTP 客户端（前端不再直连后端，所有网络请求经由此处）。

use std::sync::{Mutex, OnceLock};

use serde_json::{json, Value};

use crate::crypt::FieldCrypt;
use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::http::HttpClient;

/// 全局共享状态
pub struct AppState {
    /// 启动阶段一次性写入，运行期只读
    database: OnceLock<Database>,
    crypt: OnceLock<FieldCrypt>,
    http: HttpClient,
    /// 启动阶段致命错误（数据层不可用时，命令层据此返回可读信息）
    startup_error: Mutex<Option<String>>,
    /// 启动自检报告（密钥来源 / 库路径 / SQLCipher 版本 / 表数量等）
    bootstrap_report: Mutex<Option<Value>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            database: OnceLock::new(),
            crypt: OnceLock::new(),
            http: HttpClient::new(),
            startup_error: Mutex::new(None),
            bootstrap_report: Mutex::new(None),
        }
    }

    /// 统一 HTTP 客户端
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// 加密数据库句柄
    pub fn database(&self) -> AppResult<&Database> {
        match self.database.get() {
            Some(database) => Ok(database),
            None => Err(AppError::unavailable(self.failure_hint("本地数据库未初始化"))),
        }
    }

    /// 字段级加解密器
    pub fn crypt(&self) -> AppResult<&FieldCrypt> {
        match self.crypt.get() {
            Some(crypt) => Ok(crypt),
            None => Err(AppError::unavailable(
                self.failure_hint("字段加解密器未初始化"),
            )),
        }
    }

    /// 启动阶段写入数据层（只允许一次）
    pub fn init_data_layer(
        &self,
        database: Database,
        crypt: FieldCrypt,
        report: Value,
    ) -> AppResult<()> {
        self.crypt
            .set(crypt)
            .map_err(|_| AppError::internal("字段加解密器已初始化，禁止重复写入"))?;
        self.database
            .set(database)
            .map_err(|_| AppError::internal("数据库已初始化，禁止重复写入"))?;
        *self.lock(&self.bootstrap_report) = Some(report);
        Ok(())
    }

    /// 记录启动阶段致命错误
    pub fn set_startup_error(&self, message: impl Into<String>) {
        *self.lock(&self.startup_error) = Some(message.into());
    }

    /// 启动自检报告
    pub fn bootstrap_report(&self) -> Value {
        self.lock(&self.bootstrap_report)
            .clone()
            .unwrap_or_else(|| json!({ "ready": false }))
    }

    /// 数据层是否已就绪
    pub fn ready(&self) -> bool {
        self.database.get().is_some() && self.crypt.get().is_some()
    }

    fn lock<'a, T>(&self, mutex: &'a Mutex<T>) -> std::sync::MutexGuard<'a, T> {
        mutex.lock().unwrap_or_else(|err| err.into_inner())
    }

    fn failure_hint(&self, base: &str) -> String {
        match self.lock(&self.startup_error).clone() {
            Some(error) => format!("{base}（启动失败：{error}）"),
            None => base.to_string(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
