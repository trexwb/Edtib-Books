//! 数据访问执行层：查询 / 计数 / 写入 / 更新 / 删除。
//!
//! 行为对齐老项目 model 层（base.ts），其中 filters / order 的构造见 `query.rs`，
//! 字段 cast 见 `row.rs`。

use rusqlite::{params_from_iter, types::Value as SqlValue, Connection};
use serde_json::{Map, Value};

use super::models::ModelDef;
use super::query::{
    build_products_where, build_where, json_to_sql, placeholders, quote_ident, Where,
};
use super::row::{cast_set, process_row, raw_row};
use crate::crypt::FieldCrypt;
use crate::error::{AppError, AppResult};

/// 写入模式
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    Create,
    Update,
}

/// 当前时间（与 Knex `fn.now()` 一致：UTC 的 'YYYY-MM-DD HH:MM:SS'）
pub fn now_string() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 按表类型构造 WHERE（products 使用专用条件，其余走通用条件）
pub fn build_filters(model: &ModelDef, filters: &Value, out: &mut Where) -> AppResult<()> {
    if model.table == "products" {
        build_products_where(filters, out)
    } else {
        build_where(model, filters, out)
    }
}

/// 查询多行
pub fn select_rows(
    conn: &Connection,
    model: &ModelDef,
    filters: &Value,
    order: Option<&Value>,
    limit: Option<i64>,
    offset: Option<i64>,
    crypt: &FieldCrypt,
) -> AppResult<Vec<Value>> {
    let mut where_clause = Where::new();
    build_filters(model, filters, &mut where_clause)?;

    let mut sql = format!(
        "SELECT * FROM {} {}",
        quote_ident(model.table)?,
        where_clause.to_sql()
    );
    let order_sql = match order {
        Some(value) => super::query::order_for_get_list(model, Some(value))?,
        None => super::query::order_for_find_all(model)?,
    };
    if !order_sql.is_empty() {
        sql.push_str(&format!(" ORDER BY {order_sql}"));
    }
    if let Some(limit) = limit {
        sql.push_str(&format!(" LIMIT {limit}"));
        if let Some(offset) = offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
    } else if let Some(offset) = offset {
        sql.push_str(&format!(" LIMIT -1 OFFSET {offset}"));
    }

    let mut stmt = conn.prepare(&sql)?;
    let names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(|name| name.to_string())
        .collect();
    let mut rows = stmt.query(params_from_iter(where_clause.params.iter()))?;
    let mut list: Vec<Value> = Vec::new();
    while let Some(row) = rows.next()? {
        let map = raw_row(&names, row)?;
        list.push(process_row(model, map, crypt));
    }
    Ok(list)
}

/// 查询单行（无命中返回 null）
pub fn find_one_row(
    conn: &Connection,
    model: &ModelDef,
    filters: &Value,
    crypt: &FieldCrypt,
) -> AppResult<Value> {
    let mut list = select_rows(conn, model, filters, None, Some(1), None, crypt)?;
    Ok(list.pop().unwrap_or(Value::Null))
}

/// 计数（分页总数使用）
pub fn count_rows(conn: &Connection, model: &ModelDef, filters: &Value) -> AppResult<i64> {
    let mut where_clause = Where::new();
    build_filters(model, filters, &mut where_clause)?;
    let sql = format!(
        "SELECT COUNT(*) FROM {} {}",
        quote_ident(model.table)?,
        where_clause.to_sql()
    );
    let count: i64 = conn.query_row(&sql, params_from_iter(where_clause.params.iter()), |row| {
        row.get(0)
    })?;
    Ok(count)
}

/// 收集写入字段（对应 base.ts#save / create / update 的字段白名单逻辑）
fn collect_write_fields(
    model: &ModelDef,
    data: &Value,
    mode: WriteMode,
) -> AppResult<Map<String, Value>> {
    let map = data
        .as_object()
        .ok_or_else(|| AppError::bad_request("写入数据必须是对象"))?;
    let mut fields = Map::new();
    if model.fillable.is_empty() {
        // 未登记元数据的扩展表：按传入字段直写
        for (key, value) in map {
            if mode == WriteMode::Update && model.guarded.contains(&key.as_str()) {
                continue;
            }
            fields.insert(key.clone(), value.clone());
        }
    } else {
        for field in model.writable_fields() {
            let Some(value) = map.get(field) else { continue };
            if mode == WriteMode::Update && model.guarded.contains(&field) {
                continue;
            }
            fields.insert(field.to_string(), value.clone());
        }
    }
    Ok(fields)
}

/// 时间字段注入：create 写 created_at / updated_at，update 写 updated_at
fn inject_timestamps(model: &ModelDef, fields: &mut Map<String, Value>, mode: WriteMode) {
    let writable = model.writable_fields();
    let mut targets: Vec<&str> = vec!["updated_at"];
    if mode == WriteMode::Create {
        targets.push("created_at");
    }
    let now = now_string();
    for target in targets {
        let is_writable = model.fillable.is_empty() || writable.contains(&target);
        if !is_writable {
            continue;
        }
        fields.insert(target.to_string(), Value::String(now.clone()));
    }
}

/// 生成字段参数（含 cast set）
fn field_params(model: &ModelDef, fields: &Map<String, Value>, crypt: &FieldCrypt) -> Vec<SqlValue> {
    fields
        .iter()
        .map(|(key, value)| {
            let converted = match model.cast(key) {
                Some(kind) => cast_set(kind, value, crypt),
                None => value.clone(),
            };
            json_to_sql(&converted)
        })
        .collect()
}

/// 插入（单条或多条），返回主键数组（与老实现 knex insert 返回一致）
pub fn insert_rows(
    conn: &Connection,
    model: &ModelDef,
    rows: &[Value],
    crypt: &FieldCrypt,
) -> AppResult<Value> {
    let mut keys: Vec<Value> = Vec::new();
    for data in rows {
        let mut fields = collect_write_fields(model, data, WriteMode::Create)?;
        inject_timestamps(model, &mut fields, WriteMode::Create);
        if fields.is_empty() {
            return Err(AppError::bad_request("没有可写入的字段"));
        }
        let columns: Vec<String> = fields
            .keys()
            .map(|key| quote_ident(key))
            .collect::<AppResult<Vec<_>>>()?;
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quote_ident(model.table)?,
            columns.join(", "),
            placeholders(fields.len())
        );
        let params = field_params(model, &fields, crypt);
        conn.execute(&sql, params_from_iter(params.iter()))?;

        let key = fields
            .get(model.primary_key)
            .cloned()
            .filter(|value| !value.is_null())
            .unwrap_or_else(|| Value::Number(conn.last_insert_rowid().into()));
        keys.push(key);
    }
    Ok(Value::Array(keys))
}

/// 更新（返回受影响行数）
pub fn update_rows(
    conn: &Connection,
    model: &ModelDef,
    filters: &Value,
    data: &Value,
    crypt: &FieldCrypt,
) -> AppResult<i64> {
    let mut fields = collect_write_fields(model, data, WriteMode::Update)?;
    inject_timestamps(model, &mut fields, WriteMode::Update);
    if fields.is_empty() {
        return Ok(0);
    }
    let mut where_clause = Where::new();
    build_filters(model, filters, &mut where_clause)?;
    if where_clause.is_empty() {
        return Err(AppError::bad_request("更新条件不能为空"));
    }
    let assignments: Vec<String> = fields
        .keys()
        .map(|key| Ok(format!("{} = ?", quote_ident(key)?)))
        .collect::<AppResult<Vec<_>>>()?;
    let sql = format!(
        "UPDATE {} SET {} {}",
        quote_ident(model.table)?,
        assignments.join(", "),
        where_clause.to_sql()
    );
    let mut params = field_params(model, &fields, crypt);
    params.extend(where_clause.params.clone());
    let affected = conn.execute(&sql, params_from_iter(params.iter()))?;
    Ok(affected as i64)
}

/// 删除（返回受影响行数；条件为空时不执行，与老实现一致）
pub fn delete_rows(conn: &Connection, model: &ModelDef, filters: &Value) -> AppResult<Value> {
    let mut where_clause = Where::new();
    build_filters(model, filters, &mut where_clause)?;
    if where_clause.is_empty() {
        return Ok(Value::Null);
    }
    let sql = format!(
        "DELETE FROM {} {}",
        quote_ident(model.table)?,
        where_clause.to_sql()
    );
    let affected = conn.execute(&sql, params_from_iter(where_clause.params.iter()))?;
    Ok(Value::Number((affected as i64).into()))
}
