//! SQL 条件构造（等价迁移老项目 `model/base.ts#buildWhere` 与 `model/products.ts#buildWhere`）。
//!
//! 覆盖能力：
//!   * 通用条件：等值 / null / 数组(IN) / 操作符对象（not、like、in、notIn、>、<、>=、<=、!=
//!     、between、notBetween、isNull、isNotNull）
//!   * products 表专用条件：id.not / id.eq、standard_id、status、keywords 三级关键字策略、
//!     names、detail、category_id / shape_id（json_each 数组包含）、updated_at='today'
//!
//! 安全说明：所有字段名均经过 `quote_ident` 白名单校验（仅允许字母/数字/下划线及 `表.字段`），
//! 值一律走参数绑定，杜绝 SQL 注入。

use rusqlite::types::Value as SqlValue;
use serde_json::{Number, Value};

use crate::error::{AppError, AppResult};

use super::models::ModelDef;

/// 与老实现一致的默认分页参数
pub const DEFAULT_LIMIT: i64 = 10;
/// 与老实现一致的分页上限
pub const MAX_LIMIT: i64 = 1000;

// ---------------------------------------------------------------------------
// 值转换
// ---------------------------------------------------------------------------

/// JSON 值 -> SQLite 值
pub fn json_to_sql(value: &Value) -> SqlValue {
    match value {
        Value::Null => SqlValue::Null,
        Value::Bool(flag) => SqlValue::Integer(if *flag { 1 } else { 0 }),
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                SqlValue::Integer(int)
            } else if let Some(float) = number.as_f64() {
                SqlValue::Real(float)
            } else {
                SqlValue::Null
            }
        }
        Value::String(text) => SqlValue::Text(text.clone()),
        other => SqlValue::Text(other.to_string()),
    }
}

/// SQLite 值 -> JSON 值（Blob 以 hex 字符串表示）
pub fn sql_to_json(value: rusqlite::types::ValueRef<'_>) -> Value {
    match value {
        rusqlite::types::ValueRef::Null => Value::Null,
        rusqlite::types::ValueRef::Integer(int) => Value::Number(int.into()),
        rusqlite::types::ValueRef::Real(float) => {
            Number::from_f64(float).map(Value::Number).unwrap_or(Value::Null)
        }
        rusqlite::types::ValueRef::Text(text) => {
            Value::String(String::from_utf8_lossy(text).to_string())
        }
        rusqlite::types::ValueRef::Blob(blob) => Value::String(hex::encode(blob)),
    }
}

/// 字段 / 表名安全校验 + 双引号包裹（防止注入）
pub fn quote_ident(name: &str) -> AppResult<String> {
    if name.trim().is_empty() {
        return Err(AppError::bad_request("字段名不能为空"));
    }
    let parts: Vec<&str> = name.split('.').collect();
    let valid = parts.iter().all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    });
    if !valid {
        return Err(AppError::bad_request(format!("非法字段名: {name}")));
    }
    Ok(parts
        .iter()
        .map(|part| format!("\"{part}\""))
        .collect::<Vec<_>>()
        .join("."))
}

// ---------------------------------------------------------------------------
// WHERE 构造器
// ---------------------------------------------------------------------------

/// WHERE 子句收集器
#[derive(Default, Debug)]
pub struct Where {
    parts: Vec<String>,
    pub params: Vec<SqlValue>,
}

impl Where {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, clause: impl Into<String>, params: Vec<SqlValue>) {
        self.parts.push(clause.into());
        self.params.extend(params);
    }

    /// 追加一个 AND 组（组内已自带括号）
    pub fn push_group(&mut self, group: Where) {
        if group.parts.is_empty() {
            return;
        }
        self.push(format!("({})", group.parts.join(" AND ")), group.params);
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// 生成 ` WHERE ...`（无条件时返回空串）
    pub fn to_sql(&self) -> String {
        if self.parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.parts.join(" AND "))
        }
    }
}

pub fn placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

/// 通用条件构造（对应 base.ts#buildWhere）
pub fn build_where(model: &ModelDef, filters: &Value, out: &mut Where) -> AppResult<()> {
    match filters {
        Value::Null => Ok(()),
        Value::Array(items) => {
            for item in items {
                build_where(model, item, out)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            for (key, value) in map {
                match key.as_str() {
                    "or" => build_group(model, value, "OR", out)?,
                    "and" => build_group(model, value, "AND", out)?,
                    _ => build_field(field_name(model, key), value, out)?,
                }
            }
            Ok(())
        }
        _ => Err(AppError::bad_request("filters 必须是对象或数组")),
    }
}

/// 字段名别名映射：`categoryIds` -> `categories_id` 之类的驼峰直接按原样使用
fn field_name<'a>(_model: &ModelDef, key: &'a str) -> &'a str {
    key
}

fn build_group(model: &ModelDef, value: &Value, join: &str, out: &mut Where) -> AppResult<()> {
    let items: Vec<Value> = match value {
        Value::Array(list) => list.clone(),
        other => vec![other.clone()],
    };
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<SqlValue> = Vec::new();
    for item in items {
        let mut sub = Where::new();
        build_where(model, &item, &mut sub)?;
        if !sub.parts.is_empty() {
            clauses.push(sub.parts.join(" AND "));
            params.extend(sub.params);
        }
    }
    if !clauses.is_empty() {
        out.push(format!("({})", clauses.join(&format!(" {join} "))), params);
    }
    Ok(())
}

fn build_field(field: &str, value: &Value, out: &mut Where) -> AppResult<()> {
    let column = quote_ident(field)?;
    match value {
        Value::Null => out.push(format!("{column} IS NULL"), vec![]),
        Value::Array(items) => {
            if items.is_empty() {
                out.push("1 = 0", vec![]);
            } else {
                out.push(
                    format!("{column} IN ({})", placeholders(items.len())),
                    items.iter().map(json_to_sql).collect(),
                );
            }
        }
        Value::Object(operators) => {
            for (op, op_value) in operators {
                apply_operator(&column, op, op_value, out)?;
            }
        }
        other => out.push(format!("{column} = ?"), vec![json_to_sql(other)]),
    }
    Ok(())
}

fn apply_operator(column: &str, op: &str, value: &Value, out: &mut Where) -> AppResult<()> {
    let value_list = |value: &Value| -> Option<Vec<SqlValue>> {
        value.as_array().map(|items| items.iter().map(json_to_sql).collect())
    };
    match op {
        "eq" | "=" => match value {
            Value::Null => out.push(format!("{column} IS NULL"), vec![]),
            Value::Array(_) => {
                let items = value_list(value).unwrap_or_default();
                if items.is_empty() {
                    out.push("1 = 0", vec![]);
                } else {
                    out.push(
                        format!("{column} IN ({})", placeholders(items.len())),
                        items,
                    );
                }
            }
            other => out.push(format!("{column} = ?"), vec![json_to_sql(other)]),
        },
        "not" | "neq" | "!=" | "<>" => match value {
            Value::Null => out.push(format!("{column} IS NOT NULL"), vec![]),
            Value::Array(_) => {
                let items = value_list(value).unwrap_or_default();
                if items.is_empty() {
                    out.push("1 = 1", vec![]);
                } else {
                    out.push(
                        format!("{column} NOT IN ({})", placeholders(items.len())),
                        items,
                    );
                }
            }
            other => out.push(format!("{column} != ?"), vec![json_to_sql(other)]),
        },
        "like" => out.push(format!("{column} LIKE ?"), vec![json_to_sql(value)]),
        "notLike" => out.push(format!("{column} NOT LIKE ?"), vec![json_to_sql(value)]),
        "in" => {
            let items = value_list(value).unwrap_or_else(|| vec![json_to_sql(value)]);
            out.push(
                format!("{column} IN ({})", placeholders(items.len())),
                items,
            );
        }
        "notIn" => {
            let items = value_list(value).unwrap_or_else(|| vec![json_to_sql(value)]);
            out.push(
                format!("{column} NOT IN ({})", placeholders(items.len())),
                items,
            );
        }
        ">" | "<" | ">=" | "<=" => {
            out.push(format!("{column} {op} ?"), vec![json_to_sql(value)]);
        }
        "between" | "notBetween" => {
            let items = value_list(value).unwrap_or_default();
            if items.len() == 2 {
                let keyword = if op == "between" { "BETWEEN" } else { "NOT BETWEEN" };
                out.push(
                    format!("{column} {keyword} ? AND ?"),
                    vec![items[0].clone(), items[1].clone()],
                );
            }
        }
        "isNull" | "null" => out.push(format!("{column} IS NULL"), vec![]),
        "isNotNull" | "notNull" => out.push(format!("{column} IS NOT NULL"), vec![]),
        other => {
            return Err(AppError::bad_request(format!(
                "不支持的查询操作符: {other}（字段 {column}）"
            )))
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// products 专用条件（对应 model/products.ts#buildWhere）
// ---------------------------------------------------------------------------

const PRODUCTS_KEYWORD_FALLBACK_FIELDS: &[&str] =
    &["standard", "grade", "names", "code", "year", "extension"];

enum KeywordType {
    Numbers,
    Letters,
    LettersNumbers,
    Other,
}

fn detect_keyword_type(input: &str) -> KeywordType {
    let is_pure_letters = !input.is_empty()
        && input.chars().all(|c| c.is_ascii_alphabetic());
    let is_pure_numbers = !input.is_empty() && input.chars().all(|c| c.is_ascii_digit());
    let letters_then_numbers = {
        let mut seen_digit = false;
        let mut ok = !input.is_empty();
        for ch in input.chars() {
            if ch.is_ascii_digit() {
                seen_digit = true;
            } else if ch.is_ascii_alphabetic() {
                if seen_digit {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        ok && seen_digit && input.chars().any(|c| c.is_ascii_alphabetic())
    };
    if is_pure_letters {
        KeywordType::Letters
    } else if is_pure_numbers {
        KeywordType::Numbers
    } else if letters_then_numbers {
        KeywordType::LettersNumbers
    } else {
        KeywordType::Other
    }
}

fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// products 关键字三级匹配策略（忠实迁移老实现的子查询优先级逻辑）
fn products_keywords_clause(keywords: &str) -> (String, Vec<SqlValue>) {
    match detect_keyword_type(keywords) {
        KeywordType::Numbers => {
            let prefix = format!("{keywords}%");
            let fuzzy = format!("%{keywords}%");
            let sql = r#"(
  (
    (SELECT COUNT(*) FROM "products" AS a WHERE a."code" = ? OR a."year" = ?) > 0
    AND ("code" = ? OR "year" = ?)
  ) OR (
    (SELECT COUNT(*) FROM "products" AS a WHERE a."code" = ? OR a."year" = ?) = 0
    AND (SELECT COUNT(*) FROM "products" AS b WHERE b."code" LIKE ?) > 0
    AND ("code" LIKE ?)
  ) OR (
    (SELECT COUNT(*) FROM "products" AS a WHERE a."code" = ? OR a."year" = ?) = 0
    AND (SELECT COUNT(*) FROM "products" AS b WHERE b."code" LIKE ?) = 0
    AND ("code" LIKE ? OR "year" LIKE ?)
  )
)"#;
            let params = vec![
                text(keywords),
                text(keywords),
                text(keywords),
                text(keywords),
                text(keywords),
                text(keywords),
                text(&prefix),
                text(&prefix),
                text(keywords),
                text(keywords),
                text(&prefix),
                text(&fuzzy),
                text(&fuzzy),
            ];
            (sql.to_string(), params)
        }
        KeywordType::Letters => {
            let sql = r#"(
  INSTR("standard" || CASE WHEN "grade" IS NOT NULL THEN '/' || "grade" ELSE '' END, ?) > 0
  OR INSTR("names", ?) > 0
)"#;
            let params = vec![text(keywords), text(keywords)];
            (sql.to_string(), params)
        }
        KeywordType::LettersNumbers => {
            let letters: String = keywords.chars().filter(|c| c.is_ascii_alphabetic()).collect();
            let numbers: String = keywords.chars().filter(|c| c.is_ascii_digit()).collect();
            if letters.is_empty() || numbers.is_empty() {
                return products_keywords_fallback(keywords);
            }
            let prefix = format!("{numbers}%");
            let letters_sql = r#"INSTR("standard" || COALESCE("grade", ''), ?)"#;
            let sql = format!(
                r#"(
  (
    (SELECT COUNT(*) FROM "products" AS a WHERE (a."code" = ? OR a."year" = ?) AND (INSTR(a."standard" || COALESCE(a."grade", ''), ?) > 0)) > 0
    AND (("code" = ? OR "year" = ?) AND ({letters_inner}) > 0)
  ) OR (
    (SELECT COUNT(*) FROM "products" AS a WHERE (a."code" = ? OR a."year" = ?) AND (INSTR(a."standard" || COALESCE(a."grade", ''), ?) > 0)) = 0
    AND (SELECT COUNT(*) FROM "products" AS b WHERE (b."code" LIKE ?) AND (INSTR(b."standard" || COALESCE(b."grade", ''), ?) > 0)) > 0
    AND ("code" LIKE ?)
    AND ({letters_inner}) > 0
  ) OR (
    (SELECT COUNT(*) FROM "products" AS a WHERE (a."code" = ? OR a."year" = ?) AND (INSTR(a."standard" || COALESCE(a."grade", ''), ?) > 0)) = 0
    AND (SELECT COUNT(*) FROM "products" AS b WHERE (b."code" LIKE ?) AND (INSTR(b."standard" || COALESCE(b."grade", ''), ?) > 0)) = 0
    AND (INSTR("code", ?) > 0 OR INSTR("year", ?) > 0)
    AND (INSTR("standard" || CASE WHEN "grade" IS NOT NULL THEN '/' || "grade" ELSE '' END, ?) > 0)
  )
)"#,
                letters_inner = letters_sql
            );
            let params = vec![
                text(&numbers),
                text(&numbers),
                text(&letters),
                text(&numbers),
                text(&numbers),
                text(&letters),
                text(&numbers),
                text(&numbers),
                text(&letters),
                text(&prefix),
                text(&letters),
                text(&prefix),
                text(&letters),
                text(&numbers),
                text(&numbers),
                text(&letters),
                text(&prefix),
                text(&letters),
                text(&numbers),
                text(&numbers),
                text(&letters),
            ];
            (sql, params)
        }
        KeywordType::Other => products_keywords_fallback(keywords),
    }
}

fn products_keywords_fallback(keywords: &str) -> (String, Vec<SqlValue>) {
    let sql = format!(
        "({})",
        PRODUCTS_KEYWORD_FALLBACK_FIELDS
            .iter()
            .map(|field| format!("INSTR(\"{field}\", ?) > 0"))
            .collect::<Vec<_>>()
            .join(" OR ")
    );
    let params = PRODUCTS_KEYWORD_FALLBACK_FIELDS
        .iter()
        .map(|_| text(keywords))
        .collect();
    (sql, params)
}

/// json_each 数组包含查询（categories_id / shapes_id）
fn json_contains_clause(column: &str, value: i64) -> (String, Vec<SqlValue>) {
    let sql = format!(
        "\"id\" IN (SELECT \"products\".\"id\" FROM \"products\", json_each(\"products\".\"{column}\") WHERE json_each.value = ?)"
    );
    (sql, vec![SqlValue::Integer(value)])
}

fn to_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// products 表条件（其余字段仍走通用构造）
pub fn build_products_where(filters: &Value, out: &mut Where) -> AppResult<()> {
    // 老实现固定附加 id > 0
    out.push("\"id\" > 0", vec![]);

    let map = match filters {
        Value::Null => return Ok(()),
        Value::Array(items) => {
            // 数组形式：逐个套用
            for item in items {
                build_products_where(item, out)?;
            }
            return Ok(());
        }
        Value::Object(map) => map.clone(),
        _ => return Err(AppError::bad_request("filters 必须是对象或数组")),
    };

    // id / id.not / id.eq
    if let Some(id_value) = map.get("id") {
        if let Value::Object(operators) = id_value {
            if let Some(not_value) = operators.get("not") {
                match not_value {
                    Value::Array(items) if !items.is_empty() => out.push(
                        format!("\"id\" NOT IN ({})", placeholders(items.len())),
                        items.iter().map(json_to_sql).collect(),
                    ),
                    Value::Null => {}
                    other => out.push("\"id\" != ?".to_string(), vec![json_to_sql(other)]),
                }
            }
            if let Some(eq_value) = operators.get("eq") {
                match eq_value {
                    Value::Array(items) if !items.is_empty() => out.push(
                        format!("\"id\" IN ({})", placeholders(items.len())),
                        items.iter().map(json_to_sql).collect(),
                    ),
                    Value::Null => {}
                    other => out.push("\"id\" = ?".to_string(), vec![json_to_sql(other)]),
                }
            }
        } else if !id_value.is_null() {
            match id_value {
                Value::Array(items) if !items.is_empty() => out.push(
                    format!("\"id\" IN ({})", placeholders(items.len())),
                    items.iter().map(json_to_sql).collect(),
                ),
                Value::Array(_) => {}
                other => out.push("\"id\" = ?".to_string(), vec![json_to_sql(other)]),
            }
        }
    }

    if let Some(value) = map.get("standard_id") {
        if truthy(value) {
            value_clause("standard_id", value, out)?;
        }
    }

    if let Some(value) = map.get("status") {
        if truthy(value) || value.as_str() == Some("0") {
            value_clause("status", value, out)?;
        }
    }

    if let Some(Value::String(keywords)) = map.get("keywords") {
        let cleaned: String = keywords.chars().filter(|c| !c.is_whitespace()).collect();
        if !cleaned.is_empty() {
            // 注意：检测与参数均使用原始关键字（与老实现一致），仅空值判断使用去空白后的串
            let (clause, params) = products_keywords_clause(keywords);
            out.push(clause, params);
        }
    }

    for field in ["names", "detail"] {
        if let Some(Value::String(value)) = map.get(field) {
            if !value.is_empty() {
                out.push(format!("INSTR(\"{field}\", ?) > 0"), vec![text(value)]);
            }
        }
    }

    for (field, column) in [("category_id", "categories_id"), ("shape_id", "shapes_id")] {
        if let Some(value) = map.get(field) {
            match value {
                Value::Array(items) if !items.is_empty() => {
                    let last = &items[items.len() - 1];
                    if let Some(number) = to_number(last) {
                        let (clause, params) = json_contains_clause(column, number as i64);
                        out.push(clause, params);
                    }
                }
                Value::Array(_) => {}
                other => {
                    if let Some(number) = to_number(other) {
                        if number > 0.0 {
                            let (clause, params) = json_contains_clause(column, number as i64);
                            out.push(clause, params);
                        }
                    }
                }
            }
        }
    }

    if let Some(Value::String(updated_at)) = map.get("updated_at") {
        if updated_at == "today" {
            out.push("\"updated_at\" < ?".to_string(), vec![text(&today_iso())]);
        }
    }

    // 其余字段走通用条件
    let handled = [
        "id",
        "standard_id",
        "status",
        "keywords",
        "names",
        "detail",
        "category_id",
        "shape_id",
        "updated_at",
    ];
    for (key, value) in &map {
        if handled.contains(&key.as_str()) {
            continue;
        }
        build_field(key, value, out)?;
    }

    Ok(())
}

fn value_clause(field: &str, value: &Value, out: &mut Where) -> AppResult<()> {
    let column = quote_ident(field)?;
    match value {
        Value::Array(items) if !items.is_empty() => {
            out.push(
                format!("{column} IN ({})", placeholders(items.len())),
                items.iter().map(json_to_sql).collect(),
            );
            Ok(())
        }
        Value::Array(_) => Ok(()),
        other => {
            out.push(format!("{column} = ?"), vec![json_to_sql(other)]);
            Ok(())
        }
    }
}

/// JS 真值判断（0 / "" / false / null / undefined 之外为真）
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

/// 今日零点的 ISO 字符串（等价 `new Date().setHours(0,0,0,0).toISOString()`）
fn today_iso() -> String {
    use chrono::{Local, SecondsFormat, TimeZone, Utc};
    let now = Local::now();
    let naive = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let midnight = match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => now,
    };
    midnight
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

// ---------------------------------------------------------------------------
// ORDER BY 构造
// ---------------------------------------------------------------------------

/// 递归判断 order 中是否出现 "sort"（对应老实现 containsSort）
pub fn contains_sort(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains("sort"),
        Value::Array(items) => items.iter().any(contains_sort),
        Value::Object(map) => map.values().any(contains_sort),
        _ => false,
    }
}

/// 解析 order 参数：数组（或 JSON 字符串）
pub fn parse_order(order: &Value) -> Vec<(String, String)> {
    let items: Vec<Value> = match order {
        Value::Array(items) => items.clone(),
        Value::String(text) => serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| {
            let column = item.get("column").and_then(|c| c.as_str())?;
            let direction = item
                .get("order")
                .and_then(|o| o.as_str())
                .unwrap_or("ASC")
                .to_ascii_uppercase();
            let direction = if direction == "DESC" { "DESC" } else { "ASC" };
            Some((column.to_string(), direction.to_string()))
        })
        .collect()
}

/// 生成 ORDER BY 片段（不含 " ORDER BY "）
///
/// * `with_sort_rule`：老实现中 getAll 恒附加，getList 仅在 order 含 sort 时附加
/// * `orders`：为空且 `with_sort_rule` 为 false 时使用主键升序
fn order_clause(
    orders: &[(String, String)],
    include_sort_rule: bool,
    primary_key: &str,
    default_primary: bool,
) -> AppResult<String> {
    let mut parts: Vec<String> = Vec::new();
    if include_sort_rule {
        parts.push("CASE WHEN \"sort\" > 0 THEN 1 ELSE 0 END DESC".to_string());
        parts.push("\"sort\" ASC".to_string());
    }
    for (column, direction) in orders {
        parts.push(format!("{} {direction}", quote_ident(column)?));
    }
    if parts.is_empty() && default_primary {
        parts.push(format!("{} ASC", quote_ident(primary_key)?));
    }
    Ok(parts.join(", "))
}

/// getAll 的排序：字段包含 sort 时附加 sort 规则（老实现恒附加）
pub fn order_for_find_all(model: &ModelDef) -> AppResult<String> {
    let has_sort = model.sortable_fields().contains(&"sort");
    order_clause(&[], has_sort, model.primary_key, false)
}

/// getList 的排序
pub fn order_for_get_list(model: &ModelDef, order: Option<&Value>) -> AppResult<String> {
    let orders = match order {
        None => Vec::new(),
        Some(value) => parse_order(value),
    };
    let include_sort_rule = order
        .map(|value| contains_sort(value))
        .unwrap_or(false);
    // order 为 None 时使用主键升序；显式传入空数组时不排序（与老实现一致）
    order_clause(&orders, include_sort_rule, model.primary_key, order.is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_keyword_types() {
        assert!(matches!(detect_keyword_type("2025"), KeywordType::Numbers));
        assert!(matches!(detect_keyword_type("GB"), KeywordType::Letters));
        assert!(matches!(
            detect_keyword_type("GB9074"),
            KeywordType::LettersNumbers
        ));
        assert!(matches!(detect_keyword_type("GB-9074"), KeywordType::Other));
    }

    #[test]
    fn quote_ident_blocks_injection() {
        assert!(quote_ident("id").is_ok());
        assert!(quote_ident("products.id").is_ok());
        assert!(quote_ident("id; DROP TABLE users").is_err());
    }
}
