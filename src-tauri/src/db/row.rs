//! 行数据转换：字段值 cast（对应老项目 cast/*.ts）+ 时间字段格式化。
//!
//! 与老实现的对应关系逐条迁移，差异见各函数注释。

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rusqlite::Row;
use serde_json::{Map, Number, Value};

use super::models::{CastKind, ModelDef};
use super::query::sql_to_json;
use crate::crypt::FieldCrypt;

/// 老实现 `formatTimestamps` 处理的时间字段
pub const TIMESTAMP_FIELDS: [&str; 4] = ["created_at", "updated_at", "deleted_at", "times_expire"];

/// 数据库时间格式（与 dayjs 的 'YYYY-MM-DD HH:mm:ss' 一致）
const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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
    for field in TIMESTAMP_FIELDS {
        if let Some(value) = result.get(field).cloned() {
            if !value.is_null() {
                let formatted = format_datetime(&value);
                result.insert(field.to_string(), formatted);
            }
        }
    }
    Value::Object(result)
}

// ---------------------------------------------------------------------------
// 读取方向（数据库 -> 前端）
// ---------------------------------------------------------------------------

/// cast/integer.ts：parseInt，非法值取 0
fn cast_integer(value: Value) -> Value {
    match value {
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                Value::Number(int.into())
            } else {
                Value::Number((number.as_f64().unwrap_or(0.0) as i64).into())
            }
        }
        Value::String(text) => Value::Number(parse_int_or_zero(&text).into()),
        Value::Bool(flag) => Value::Number(if flag { 1 } else { 0 }.into()),
        _ => Value::Number(0.into()),
    }
}

/// cast/integerOrNull.ts：空 / 非法值取 null
fn cast_integer_or_null(value: Value) -> Value {
    match value {
        Value::Null => Value::Null,
        Value::String(text) if text.trim().is_empty() => Value::Null,
        Value::String(text) => match text.trim().parse::<i64>() {
            Ok(int) => Value::Number(int.into()),
            Err(_) => Value::Null,
        },
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                Value::Number(int.into())
            } else {
                match number.as_f64() {
                    Some(float) if !float.is_nan() => Value::Number((float as i64).into()),
                    _ => Value::Null,
                }
            }
        }
        Value::Bool(flag) => Value::Number(if flag { 1 } else { 0 }.into()),
        _ => Value::Null,
    }
}

fn parse_int_or_zero(text: &str) -> i64 {
    let trimmed = text.trim();
    let digits: String = trimmed
        .trim_start_matches(['+', '-'])
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return 0;
    }
    let negative = trimmed.starts_with('-');
    let magnitude: i64 = digits.parse().unwrap_or(0);
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// cast/string.ts：false / null 原样保留，其余 toString（falsy -> ''）
fn cast_string(value: Value) -> Value {
    match value {
        Value::Null | Value::Bool(false) => value,
        Value::String(text) => Value::String(text),
        Value::Bool(true) => Value::String("true".to_string()),
        Value::Number(number) => {
            if number.as_f64() == Some(0.0) {
                Value::String(String::new())
            } else {
                Value::String(number.to_string())
            }
        }
        Value::Array(_) | Value::Object(_) => Value::String(value.to_string()),
    }
}

/// cast/json.ts：解析失败时返回空对象 {}
fn cast_json(value: Value) -> Value {
    match &value {
        Value::String(text) => {
            if text.trim().is_empty() {
                return Value::Object(Map::new());
            }
            serde_json::from_str::<Value>(text).unwrap_or_else(|_| Value::Object(Map::new()))
        }
        Value::Null => Value::Object(Map::new()),
        other => other.clone(),
    }
}

/// cast/datetime.ts：dayjs(value).format('YYYY-MM-DD HH:mm:ss')
fn cast_datetime(value: Value) -> Value {
    if is_falsy(&value) {
        return Value::Null;
    }
    match format_datetime(&value) {
        Value::String(text) => Value::String(text),
        _ => Value::String("Invalid Date".to_string()),
    }
}

/// cast/boolean.ts：字符串仅 '1' / 'true' / 'yes' / 'on' 为真
fn cast_boolean(value: Value) -> Value {
    match value {
        Value::Bool(flag) => Value::Bool(flag),
        Value::Number(number) => Value::Bool(number.as_f64() == Some(1.0)),
        Value::String(text) => {
            let lower = text.to_lowercase();
            Value::Bool(matches!(lower.as_str(), "1" | "true" | "yes" | "on"))
        }
        other => Value::Bool(!is_falsy(&other)),
    }
}

/// 字段读转换
pub fn cast_get(kind: CastKind, value: Value, crypt: &FieldCrypt) -> Value {
    match kind {
        CastKind::Integer => cast_integer(value),
        CastKind::IntegerOrNull => cast_integer_or_null(value),
        CastKind::String => cast_string(value),
        CastKind::Json => cast_json(value),
        CastKind::Datetime => cast_datetime(value),
        CastKind::Boolean => cast_boolean(value),
        CastKind::Crypt => match value {
            // 未配置业务密钥时透传密文（见 crypt.rs 说明）
            Value::String(text) => crypt
                .decrypt_value(&text)
                .unwrap_or(Value::String(text)),
            other => other,
        },
    }
}

// ---------------------------------------------------------------------------
// 写入方向（前端 -> 数据库）
// ---------------------------------------------------------------------------

/// 字段写转换（对应 cast#set）
pub fn cast_set(kind: CastKind, value: &Value, crypt: &FieldCrypt) -> Value {
    match kind {
        // set：number 原样；字符串 parseFloat（NaN -> 0）；其余 0
        CastKind::Integer => match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|float| !float.is_nan())
                .map(|float| Value::Number(float_to_number(float)))
                .unwrap_or_else(|| Value::Number(0.into())),
            Value::String(text) if !text.trim().is_empty() => text
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|float| !float.is_nan())
                .map(|float| Value::Number(float_to_number(float)))
                .unwrap_or_else(|| Value::Number(0.into())),
            _ => Value::Number(0.into()),
        },
        // set：null / undefined -> null；Number(value) NaN -> null
        CastKind::IntegerOrNull => match value {
            Value::Null => Value::Null,
            Value::String(text) if text.trim().is_empty() => Value::Null,
            Value::Number(number) => {
                if number.as_f64().map(|f| f.is_nan()).unwrap_or(true) {
                    Value::Null
                } else {
                    value.clone()
                }
            }
            Value::String(text) => match text.trim().parse::<f64>() {
                Ok(float) if !float.is_nan() => Value::Number(float_to_number(float)),
                _ => Value::Null,
            },
            Value::Bool(flag) => Value::Number(if *flag { 1 } else { 0 }.into()),
            _ => Value::Null,
        },
        CastKind::String => cast_string(value.clone()),
        // set：字符串原样（保留已序列化内容），其余 JSON.stringify；失败 -> "{}"
        CastKind::Json => match value {
            Value::String(text) => Value::String(text.clone()),
            other => Value::String(
                serde_json::to_string(other).unwrap_or_else(|_| "{}".to_string()),
            ),
        },
        CastKind::Datetime => {
            if is_falsy(value) {
                Value::Null
            } else {
                match format_datetime(value) {
                    Value::String(text) => Value::String(text),
                    _ => Value::Null,
                }
            }
        }
        CastKind::Boolean => {
            let truthy = match value {
                Value::Bool(flag) => *flag,
                _ => !is_falsy(value),
            };
            Value::String(if truthy { "1" } else { "0" }.to_string())
        }
        CastKind::Crypt => {
            if is_falsy(value) {
                Value::Null
            } else {
                crypt
                    .encrypt_value(value)
                    .map(Value::String)
                    .unwrap_or_else(|| value.clone())
            }
        }
    }
}

fn float_to_number(float: f64) -> Number {
    if float.fract() == 0.0 && float.abs() < 9_007_199_254_740_992.0 {
        Number::from(float as i64)
    } else {
        Number::from_f64(float).unwrap_or_else(|| Number::from(0))
    }
}

/// JS falsy 判断：null / false / 0 / "" / [] / {}
pub fn is_falsy(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(flag) => !flag,
        Value::Number(number) => number.as_f64() == Some(0.0),
        Value::String(text) => text.is_empty(),
        // JS 中空数组 / 空对象为真值，此处按真值处理
        Value::Array(_) | Value::Object(_) => false,
    }
}

// ---------------------------------------------------------------------------
// 时间格式化
// ---------------------------------------------------------------------------

/// 判断字符串是否已是 `YYYY-MM-DD HH:MM:SS`（含毫秒 / T 分隔符变体）
fn is_sql_datetime_prefix(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 19 {
        return false;
    }
    let digits = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18];
    digits.iter().all(|i| bytes[*i].is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && (bytes[10] == b' ' || bytes[10] == b'T')
        && bytes[13] == b':'
        && bytes[16] == b':'
}

/// dayjs 等价格式化：输出 'YYYY-MM-DD HH:mm:ss'
pub fn format_datetime(value: &Value) -> Value {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Value::Null;
            }
            if is_sql_datetime_prefix(trimmed) {
                let normalized = format!("{} {}", &trimmed[..10], &trimmed[11..19]);
                return Value::String(normalized);
            }
            // ISO8601 / 带时区
            if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
                return Value::String(
                    parsed
                        .with_timezone(&Local)
                        .format(DATETIME_FORMAT)
                        .to_string(),
                );
            }
            // 纯日期
            if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
                return Value::String(
                    date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .format(DATETIME_FORMAT)
                        .to_string(),
                );
            }
            // 无时区的 ISO（按本地时间解析，与 dayjs 一致）
            if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.f") {
                return Value::String(naive.format(DATETIME_FORMAT).to_string());
            }
            Value::Null
        }
        Value::Number(number) => {
            let Some(raw) = number.as_f64() else {
                return Value::Null;
            };
            if raw.is_nan() || raw.is_infinite() {
                return Value::Null;
            }
            // 秒 / 毫秒自适应
            let millis = if raw.abs() < 100_000_000_000.0 {
                (raw * 1000.0) as i64
            } else {
                raw as i64
            };
            match Utc.timestamp_millis_opt(millis).single() {
                Some(datetime) => Value::String(
                    datetime
                        .with_timezone(&Local)
                        .format(DATETIME_FORMAT)
                        .to_string(),
                ),
                None => Value::Null,
            }
        }
        _ => Value::Null,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_passthrough_and_convert() {
        assert_eq!(
            format_datetime(&Value::String("2025-07-21 19:18:34".into())),
            Value::String("2025-07-21 19:18:34".into())
        );
        assert_eq!(
            format_datetime(&Value::String("2025-07-21T19:18:34.123Z".into()))
                .as_str()
                .unwrap()
                .len(),
            19
        );
    }

    #[test]
    fn integer_set_matches_legacy() {
        let crypt = FieldCrypt::from_secret(None, None);
        assert_eq!(
            cast_set(CastKind::Integer, &Value::String("12.7".into()), &crypt),
            Value::Number(12.7f64.into())
        );
        assert_eq!(
            cast_set(CastKind::Integer, &Value::String("abc".into()), &crypt),
            Value::Number(0.into())
        );
    }
}
