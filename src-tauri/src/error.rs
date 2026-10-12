//! 统一错误类型。
//!
//! 与老 Electron 主进程的返回协议保持一致：`{ code, message, timestamp, data }`。
//! 旧的 `ipcMain.handle` 在异常时返回 `code: 500` 的信封（而不是抛出），
//! 前端 `requestBridge` 依赖 `code === 200` / `code !== 200` 判断是否降级为 HTTP。
//! 成功信封的构造统一收敛到 [`crate::response`]，本模块只负责错误侧：
//!   * 数据类命令（db / fs）异常时返回信封错误（本模块的 `AppError` 序列化即为信封）；
//!   * 前端调用点内部再按 `code` 做分支（见 src/bridge/index.ts）。

use std::fmt;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json::Value;

pub type AppResult<T> = Result<T, AppError>;

/// 业务错误：序列化后即为老 Electron 的响应信封（data 固定为 null）。
#[derive(Debug, Clone)]
pub struct AppError {
    pub code: i64,
    pub message: String,
}

impl AppError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// 400 客户端参数错误
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    /// 403 越权 / 非法路径
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403, message)
    }

    /// 404 资源不存在
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    /// 500 内部错误（与旧实现一致的默认码）
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }

    /// 503 能力不可用（如数据库未初始化）
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(503, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 4)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("timestamp", &crate::js::now_iso())?;
        state.serialize_field("data", &Value::Null)?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::internal(format!("数据库错误: {err}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::internal(format!("文件操作失败: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::internal(format!("JSON 解析失败: {err}"))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::internal(format!("HTTP 请求失败: {err}"))
    }
}
