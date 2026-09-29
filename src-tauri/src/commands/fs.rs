//! 文件系统命令（对应 `src/bridge/channels.ts` 的 `fs.*`，实现老 Electron `fs-*` IPC 语义）。
//!
//! 与老实现的差异（安全增强，路径语义保持兼容）：
//!   * 老实现无脑 `path.join(userData, filePath)`，绝对路径会直接覆盖 userData 前缀，
//!     等于允许渲染进程读写任意位置；
//!   * 新实现：相对路径一律解析到应用数据目录（等价于老 `path.join`），绝对路径仅允许落在
//!     「应用数据目录 / 系统文档目录」内，且禁止任何 `..` 跳转。
//!
//! 返回结构：`fs_read_file` / `fs_write_file` 返回统一信封（前端 `requestBridge` 读取 `code` /
//! `data`）；`fs_delete_file` / `fs_exists` / `fs_get_*_path` / `fs_list_files` 返回裸值，
//! 与 `src/bridge/types.ts` 中声明的类型（boolean / string / string[]）一致。

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use tauri::AppHandle;

use crate::error::{ok_envelope, AppError, AppResult};
use crate::paths;

/// 解析前端传入的路径。
///
/// * `empty_as_base = true` 时，空字符串等价于应用数据目录根（老实现 `path.join(userData, '')`
///   的语义，前端 `cachesClear` 会以空 dir 列出整个缓存目录）；
/// * 其余情况空字符串视为非法。
pub(crate) fn resolve_input(
    app: &AppHandle,
    input: &str,
    empty_as_base: bool,
) -> AppResult<PathBuf> {
    let trimmed = input.trim();
    let base = paths::data_dir(app);
    if trimmed.is_empty() {
        if empty_as_base {
            return Ok(base);
        }
        return Err(AppError::bad_request("路径不能为空"));
    }

    for component in Path::new(trimmed).components() {
        if matches!(component, Component::ParentDir) {
            return Err(AppError::forbidden(format!(
                "非法路径（禁止使用 .. 跳转）：{trimmed}"
            )));
        }
    }

    let raw = Path::new(trimmed);
    if raw.is_absolute() {
        let allowed = [base.clone(), paths::documents_dir(app)];
        if allowed.iter().any(|dir| raw.starts_with(dir)) {
            return Ok(raw.to_path_buf());
        }
        return Err(AppError::forbidden(format!(
            "非法路径（仅允许应用数据目录 / 文档目录内的绝对路径）：{trimmed}"
        )));
    }

    paths::resolve(&base, trimmed)
}

/// fs_read_file：读取文本文件，不存在时返回 `data: null`（老 `fs-read-file`，前端依赖该约定）
#[tauri::command]
pub async fn fs_read_file(app: AppHandle, file_path: String) -> AppResult<Value> {
    let path = resolve_input(&app, &file_path, false)?;
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(ok_envelope(Value::String(content))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(ok_envelope(Value::Null)),
        Err(err) => Err(AppError::internal(format!(
            "读取文件失败 {}：{err}",
            paths::to_string(&path)
        ))),
    }
}

/// fs_write_file：写入文本文件（自动创建父目录），返回写入的绝对路径
#[tauri::command]
pub async fn fs_write_file(
    app: AppHandle,
    file_path: String,
    data: String,
) -> AppResult<Value> {
    let path = resolve_input(&app, &file_path, false)?;
    if let Some(parent) = path.parent() {
        paths::ensure_dir(parent)?;
    }
    std::fs::write(&path, data).map_err(|err| {
        AppError::internal(format!("写入文件失败 {}：{err}", paths::to_string(&path)))
    })?;
    Ok(ok_envelope(Value::String(paths::to_string(&path))))
}

/// fs_delete_file：删除文件（老 `fs-delete-file`：失败返回 false，不抛错）
#[tauri::command]
pub async fn fs_delete_file(app: AppHandle, file_path: String) -> AppResult<bool> {
    let path = match resolve_input(&app, &file_path, false) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("[fs_delete_file] 已拒绝非法路径：{err}");
            return Ok(false);
        }
    };
    if !path.exists() {
        return Ok(false);
    }
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(err) => {
            eprintln!("[fs_delete_file] 删除失败 {}：{err}", paths::to_string(&path));
            Ok(false)
        }
    }
}

/// fs_list_files：列出目录下的文件名（老 `fs-list-files`，异常时返回空数组）
#[tauri::command]
pub async fn fs_list_files(app: AppHandle, dir: String) -> AppResult<Vec<String>> {
    let path = match resolve_input(&app, &dir, true) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("[fs_list_files] 已拒绝非法路径：{err}");
            return Ok(Vec::new());
        }
    };
    let entries = match std::fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("[fs_list_files] 读取目录失败 {}：{err}", paths::to_string(&path));
            return Ok(Vec::new());
        }
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    names.sort();
    Ok(names)
}

/// fs_get_user_data_path：应用数据目录绝对路径（老 `fs-get-user-data-path`）
#[tauri::command]
pub async fn fs_get_user_data_path(app: AppHandle) -> AppResult<String> {
    let dir = paths::data_dir(&app);
    paths::ensure_dir(&dir)?;
    Ok(paths::to_string(&dir))
}

/// fs_get_documents_path：系统文档目录绝对路径（老 `fs-get-documents-path`）
#[tauri::command]
pub async fn fs_get_documents_path(app: AppHandle) -> AppResult<String> {
    Ok(paths::to_string(&paths::documents_dir(&app)))
}

/// fs_exists：文件（或目录）是否存在（老 `fs-exists`）
#[tauri::command]
pub async fn fs_exists(app: AppHandle, file_path: String) -> AppResult<bool> {
    match resolve_input(&app, &file_path, false) {
        Ok(path) => Ok(path.exists()),
        Err(err) => {
            eprintln!("[fs_exists] 已拒绝非法路径：{err}");
            Ok(false)
        }
    }
}
