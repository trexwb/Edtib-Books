//! 路径解析：数据目录、文档目录、缓存目录、数据库文件、旧版数据目录。
//!
//! 与老 Electron 的对应关系：
//!   app.getPath('userData')    -> [`data_dir`]     （macOS: ~/Library/Application Support/<identifier>）
//!   app.getPath('documents')   -> [`documents_dir`]
//!   path.join(userData, name)  -> [`resolve`]      （额外做 `..` / 绝对路径越界防护）
//!   run_<env>.bin              -> [`db_file`]
//!
//! 旧版本（Electron）userData 目录为 `.../Application Support/Edtib Book`（package.json name=edtib_book，
//! productName=Edtib Book），[`legacy_data_dir`] 用于首次启动时搬迁旧数据（run_*.bin / cache）。

use std::path::{Component, Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

/// 开发 / 生产环境标识（与旧版 `run_development.bin` / `run_production.bin` 命名保持一致）
pub fn env_name() -> &'static str {
    if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    }
}

/// 应用数据目录（可用环境变量 `EDTIB_DATA_DIR` 覆盖，便于测试与自定义部署）
pub fn data_dir(app: &AppHandle) -> PathBuf {
    if let Ok(dir) = std::env::var("EDTIB_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| legacy_data_dir())
}

/// 旧版 Electron 的 userData 目录
pub fn legacy_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("EDTIB_LEGACY_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        return PathBuf::from(base).join("Edtib Book");
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Edtib Book");
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let base = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                format!("{home}/.config")
            });
        PathBuf::from(base).join("Edtib Book")
    }
}

/// 系统文档目录（app.getPath('documents') 等价）
pub fn documents_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .document_dir()
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join("Documents")
        })
}

/// 缓存目录（cache-file 命令的落地目录，对应旧版 `<userData>/cache`）
pub fn cache_dir(app: &AppHandle) -> PathBuf {
    data_dir(app).join("cache")
}

/// 数据库文件（与旧版一致的命名：run_development.bin / run_production.bin）
pub fn db_file(app: &AppHandle) -> PathBuf {
    data_dir(app).join(format!("run_{}.bin", env_name()))
}

/// 旧版数据库文件路径（用于首次启动搬迁旧数据）
pub fn legacy_db_file() -> PathBuf {
    legacy_data_dir().join(format!("run_{}.bin", env_name()))
}

/// 确保目录存在
pub fn ensure_dir(dir: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::internal(format!("创建目录失败 {}: {e}", dir.display())))
}

/// 把「相对路径」解析为数据目录下的绝对路径。
///
/// 安全约束（相比旧实现额外增加的防护）：
///   * 禁止 `..` 跳转（防止越出 userData 目录）；
///   * 禁止绝对路径 / 盘符（旧实现 `path.join` 会被绝对路径覆盖，从而读写任意位置）。
pub fn resolve(base: &Path, relative: &str) -> AppResult<PathBuf> {
    if relative.trim().is_empty() {
        return Err(AppError::bad_request("路径不能为空"));
    }
    let rel = Path::new(relative);
    for component in rel.components() {
        match component {
            Component::ParentDir => {
                return Err(AppError::forbidden(format!(
                    "非法路径（禁止使用 .. 跳转）：{relative}"
                )))
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::forbidden(format!(
                    "非法路径（禁止绝对路径）：{relative}"
                )))
            }
            _ => {}
        }
    }
    Ok(base.join(rel))
}

/// 统一转成字符串（跨平台分隔符由系统决定，与旧实现 path.join 的结果一致）
pub fn to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
