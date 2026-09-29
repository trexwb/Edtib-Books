//! 数据库命令（对应 `src/bridge/channels.ts` 的 `db.*`，实现老 Electron `db-*` IPC 语义）。
//!
//! 返回结构与老主进程保持一致：
//!   * `db_find_all` / `db_find_one` / `db_create` / `db_update` / `db_delete` / `db_bulk_create`
//!     → 统一信封 `{ code, message, timestamp, data }`（前端 `requestBridge` 读取 `result.data`）；
//!   * `db_get_list` → 直接返回 `{ total, list }`（老实现未加信封，分页总数依赖该结构）。
//!
//! 表名白名单来自 [`crate::db::models`]（老项目 model 元数据转译），未注册的表返回明确错误，
//! 由前端按既有逻辑降级为 HTTP 请求。

use serde_json::{json, Map, Value};
use tauri::State;

use crate::db::models::ModelDef;
use crate::db::{execute, models};
use crate::error::{ok_envelope, AppError, AppResult};
use crate::state::AppState;

/// 老实现 `base.ts` 的默认分页大小
const DEFAULT_LIMIT: i64 = 10;
/// 老实现 `base.ts` 的分页上限
const MAX_LIMIT: i64 = 1000;

/// 表名 → 模型（未注册的表直接报错）
fn model_of(table: &str) -> AppResult<&'static ModelDef> {
    models::lookup(table).ok_or_else(|| AppError::bad_request(format!("未知的数据表：{table}")))
}

/// 以主键构造唯一筛选条件（find_one / update / delete 的语义）
fn filter_by_id(model: &ModelDef, id: &Value) -> Value {
    let mut map = Map::new();
    map.insert(model.primary_key.to_string(), id.clone());
    Value::Object(map)
}

/// db_find_all：按条件取全量（老 `db-find-all`）
#[tauri::command]
pub async fn db_find_all(
    state: State<'_, AppState>,
    table: String,
    filters: Option<Value>,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let filters = filters.unwrap_or_else(|| json!({}));
    let database = state.database()?;
    let crypt = state.crypt()?;
    let rows = database.with(|conn| {
        execute::select_rows(conn, model, &filters, None, None, None, crypt)
    })?;
    Ok(ok_envelope(Value::Array(rows)))
}

/// db_get_list：分页查询（老 `db-get-list`），返回 `{ total, list }`
#[tauri::command]
pub async fn db_get_list(
    state: State<'_, AppState>,
    table: String,
    filters: Option<Value>,
    order: Option<Value>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let filters = filters.unwrap_or_else(|| json!({}));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);

    let database = state.database()?;
    let crypt = state.crypt()?;
    let total = database.with(|conn| execute::count_rows(conn, model, &filters))?;
    let list = database.with(|conn| {
        execute::select_rows(
            conn,
            model,
            &filters,
            order.as_ref(),
            Some(limit),
            Some(offset),
            crypt,
        )
    })?;

    Ok(json!({ "total": total, "list": list }))
}

/// db_find_one：按主键取单条（老 `db-find-one`）
#[tauri::command]
pub async fn db_find_one(
    state: State<'_, AppState>,
    table: String,
    id: Value,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let database = state.database()?;
    let crypt = state.crypt()?;
    let row = database.with(|conn| {
        execute::find_one_row(conn, model, &filter_by_id(model, &id), crypt)
    })?;
    Ok(ok_envelope(row))
}

/// db_create：新增单条，返回落库后的完整行（老 `db-create`）
#[tauri::command]
pub async fn db_create(
    state: State<'_, AppState>,
    table: String,
    data: Value,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let database = state.database()?;
    let crypt = state.crypt()?;
    let keys = database.with(|conn| {
        execute::insert_rows(conn, model, std::slice::from_ref(&data), crypt)
    })?;
    let key = keys.get(0).cloned().unwrap_or(Value::Null);
    let row = if key.is_null() {
        Value::Null
    } else {
        database.with(|conn| execute::find_one_row(conn, model, &filter_by_id(model, &key), crypt))?
    };
    Ok(ok_envelope(row))
}

/// db_update：按主键更新，返回更新后的行（老 `db-update`）
#[tauri::command]
pub async fn db_update(
    state: State<'_, AppState>,
    table: String,
    id: Value,
    data: Value,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let database = state.database()?;
    let crypt = state.crypt()?;
    let affected = database.with(|conn| {
        execute::update_rows(conn, model, &filter_by_id(model, &id), &data, crypt)
    })?;
    let row = if affected > 0 {
        database.with(|conn| execute::find_one_row(conn, model, &filter_by_id(model, &id), crypt))?
    } else {
        Value::Null
    };
    Ok(ok_envelope(row))
}

/// db_delete：按主键删除，返回是否删除成功（老 `db-delete`）
#[tauri::command]
pub async fn db_delete(
    state: State<'_, AppState>,
    table: String,
    id: Value,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let database = state.database()?;
    let affected = database.with(|conn| {
        execute::delete_rows(conn, model, &filter_by_id(model, &id))
    })?;
    Ok(ok_envelope(json!(affected.as_i64().unwrap_or(0) > 0)))
}

/// db_bulk_create：批量新增，返回主键数组（老 `db-bulk-create`）
#[tauri::command]
pub async fn db_bulk_create(
    state: State<'_, AppState>,
    table: String,
    data: Value,
) -> AppResult<Value> {
    let model = model_of(&table)?;
    let rows = match data {
        Value::Array(rows) => rows,
        other => vec![other],
    };
    let database = state.database()?;
    let crypt = state.crypt()?;
    let keys = database.with(|conn| execute::insert_rows(conn, model, &rows, crypt))?;
    Ok(ok_envelope(keys))
}
