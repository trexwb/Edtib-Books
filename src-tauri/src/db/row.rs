//! 行数据转换：读取原始行 → 应用模型 cast → 时间字段格式化
//! （cast 本体见 [`crate::models::cast`]，对应老项目 `model/base.ts#processRow`）。

use rusqlite::Row;
use serde_json::{Map, Value};

use crate::models::cast::{cast_get, format_datetime};
use crate::models::ModelDef;

use super::query::sql_to_json;
use crate::crypt::FieldCrypt;

/// 老实现 `formatTimestamps` 处理的时间字段
pub const TIMESTAMP_FIELDS: [&str; 4] = ["created_at", "updated_at", "deleted_at", "times_expire"];

/// 把一行原始记录读成 JSON 对象（保留列名与顺序）
pub fn raw_row(names: &[String], row: &Row<'_>) -> rusqlite::Result<Map<String, Value>> {
    let mut map = Map::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        map.insert(name.clone(), sql_to_json(row.get_ref(index)?));
    }
    Ok(map)
}

/// 行处理：过滤 hidden 字段 -> 应用 cast -> 格式化时间字段（对应 base.ts#processRow）
pub fn process_row(model: &ModelDef, mut map: Map<String, Value>, crypt: &FieldCrypt) -> Value {
    for hidden in model.hidden {
        map.remove(*hidden);
    }
    let mut result = Map::with_capacity(map.len());
    for (key, value) in map {
        let converted = match model.cast(&key) {
            Some(kind) => cast_get(kind, value, crypt),
            None => value,
        };
        result.insert(key, converted);
    }
    format_timestamps(&mut result);
    Value::Object(result)
}

/// created_at / updated_at 等时间字段的显示格式化（对应 base.ts#formatTimestamps）
pub fn format_timestamps(map: &mut Map<String, Value>) {
    for field in TIMESTAMP_FIELDS {
        if let Some(value) = map.get(field).cloned() {
            if !value.is_null() {
                map.insert(field.to_string(), format_datetime(&value));
            }
        }
    }
}
