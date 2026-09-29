//! 数据库建表 DDL（由老项目 `electron/src/migrations/*.ts` 转译生成）。
//!
//! 生成方式：`temp/transpile_schema.py`（knex migration -> SQLite DDL）。
//! 首次启动时以 `CREATE TABLE IF NOT EXISTS` 幂等执行。

/// 全部业务表（按迁移顺序）
pub const TABLES: &[&str] = &[
    "seeds",
    "configs",
    "secrets",
    "users",
    "files",
    "schedules",
    "enums",
    "variables",
    "standards",
    "shapes",
    "formulas",
    "categories",
    "products",
    "interpretations",
    "docs",
    "downloads",
    "modifications",
    "docs_hits",
    "docs_browsers",
    "docs_logs",
    "downloads_logs",
    "serials",
];

/// 建表语句（幂等，可重复执行）
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS "seeds" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT NOT NULL,
  "batch" INTEGER,
  "migration_time" TEXT
);
CREATE TABLE IF NOT EXISTS "configs" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "key" TEXT NOT NULL,
  "value" TEXT,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "secrets" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "title" TEXT NOT NULL,
  "app_id" TEXT NOT NULL,
  "app_secret" TEXT NOT NULL,
  "app_iv" TEXT NOT NULL,
  "app_url" TEXT NOT NULL,
  "extension" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "users" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "nickname" TEXT,
  "truename" TEXT,
  "email" TEXT,
  "mobile" TEXT,
  "avatar" TEXT,
  "auth_token" TEXT,
  "uuid" TEXT NOT NULL UNIQUE,
  "times_expire" TEXT,
  "credit" INTEGER,
  "extension" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "files" (
  "type" TEXT,
  "url" TEXT NOT NULL UNIQUE,
  "path" TEXT NOT NULL UNIQUE,
  "source" TEXT,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "schedules" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT NOT NULL,
  "time" TEXT NOT NULL,
  "handler" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME,
  "deleted_at" TEXT
);
CREATE TABLE IF NOT EXISTS "enums" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "enum_type" TEXT NOT NULL,
  "enum_key" TEXT NOT NULL,
  "descriptions" TEXT NOT NULL,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME,
  "deleted_at" TEXT,
  UNIQUE ("enum_type", "enum_key")
);
CREATE TABLE IF NOT EXISTS "variables" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "names" TEXT NOT NULL,
  "type" INTEGER DEFAULT 0,
  "code" TEXT NOT NULL,
  "variable" TEXT NOT NULL,
  "remarks" TEXT,
  "extension" TEXT,
  "sort" INTEGER DEFAULT 0,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "standards" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "names" TEXT NOT NULL,
  "abbreviation" TEXT NOT NULL,
  "covers" TEXT,
  "remarks" TEXT,
  "extension" TEXT,
  "total" INTEGER DEFAULT 0,
  "sort" INTEGER DEFAULT 0,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "shapes" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "location" INTEGER DEFAULT 0,
  "names" TEXT NOT NULL,
  "abbreviation" TEXT NOT NULL,
  "covers" TEXT,
  "remarks" TEXT,
  "extension" TEXT,
  "total" INTEGER DEFAULT 0,
  "sort" INTEGER DEFAULT 0,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "formulas" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "type" INTEGER DEFAULT 0,
  "shape_id" INTEGER,
  "names" TEXT NOT NULL,
  "code" TEXT NOT NULL,
  "columnar" TEXT NOT NULL,
  "remarks" TEXT,
  "extension" TEXT,
  "sort" INTEGER DEFAULT 0,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "categories" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "parent_id" INTEGER DEFAULT 0,
  "names" TEXT NOT NULL,
  "abbreviation" TEXT NOT NULL,
  "covers" TEXT,
  "remarks" TEXT,
  "extension" TEXT,
  "total" INTEGER DEFAULT 0,
  "sort" INTEGER DEFAULT 0,
  "standards" TEXT,
  "shapes" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "products" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "standard" TEXT NOT NULL,
  "standard_id" INTEGER,
  "categories" TEXT NOT NULL,
  "categories_id" TEXT NOT NULL,
  "shapes" TEXT NOT NULL,
  "shapes_id" TEXT NOT NULL,
  "names" TEXT NOT NULL,
  "grade" TEXT NOT NULL,
  "code" TEXT NOT NULL,
  "year" TEXT NOT NULL,
  "covers" TEXT,
  "svgs" TEXT,
  "renders" TEXT,
  "cads" TEXT,
  "assemblies" TEXT,
  "models" TEXT,
  "detail" TEXT,
  "parameters" TEXT,
  "tolerance" TEXT,
  "diameter_length" TEXT,
  "drawing_limit" TEXT,
  "formulas" TEXT,
  "interpretation" TEXT,
  "extension" TEXT,
  "sort" INTEGER DEFAULT 0,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "interpretations" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "product_id" INTEGER,
  "titles" TEXT NOT NULL,
  "detail" TEXT,
  "keywords" TEXT,
  "extension" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "docs" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "file_id" INTEGER,
  "title" TEXT,
  "path" TEXT,
  "detail" TEXT,
  "credit" INTEGER,
  "times_expire" TEXT,
  "extension" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "downloads" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "doc_id" INTEGER,
  "uuid" TEXT NOT NULL,
  "use_credit" INTEGER,
  "use_limit" INTEGER,
  "down_url" TEXT NOT NULL,
  "times_expire" TEXT,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "modifications" (
  "key" TEXT NOT NULL UNIQUE,
  "total" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "docs_hits" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "doc_id" INTEGER NOT NULL,
  "uuid" TEXT NOT NULL,
  "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
  "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS "docs_browsers" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "doc_id" INTEGER,
  "uuid" TEXT,
  "name" TEXT,
  "product" TEXT,
  "keywords" TEXT,
  "extension" TEXT,
  "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
  "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS "docs_logs" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "doc_id" INTEGER,
  "uuid" TEXT,
  "name" TEXT,
  "product" TEXT,
  "keywords" TEXT,
  "extension" TEXT,
  "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
  "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS "downloads_logs" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "doc_id" INTEGER,
  "uuid" TEXT,
  "name" TEXT,
  "product" TEXT,
  "keywords" TEXT,
  "extension" TEXT,
  "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
  "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS "serials" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "batch" TEXT NOT NULL,
  "code" TEXT NOT NULL UNIQUE,
  "secret" TEXT,
  "type" INTEGER DEFAULT 0,
  "level" INTEGER DEFAULT 0,
  "days" INTEGER DEFAULT 0,
  "credit" INTEGER DEFAULT 0,
  "price" INTEGER DEFAULT 0,
  "remark" TEXT,
  "extension" TEXT,
  "times_expire" TEXT,
  "uuid" TEXT,
  "status" INTEGER DEFAULT 0,
  "created_at" DATETIME,
  "updated_at" DATETIME,
  "deleted_at" TEXT
);
"#;

/// 每张表的列清单（供自检 / 调试使用）
pub const TABLE_COLUMNS: &[(&str, &[&str])] = &[
    ("seeds", &["id", "name", "batch", "migration_time"]),
    ("configs", &["id", "key", "value", "created_at", "updated_at"]),
    ("secrets", &["id", "title", "app_id", "app_secret", "app_iv", "app_url", "extension", "status", "created_at", "updated_at"]),
    ("users", &["id", "nickname", "truename", "email", "mobile", "avatar", "auth_token", "uuid", "times_expire", "credit", "extension", "status", "created_at", "updated_at"]),
    ("files", &["type", "url", "path", "source", "created_at", "updated_at"]),
    ("schedules", &["id", "name", "time", "handler", "status", "created_at", "updated_at", "deleted_at"]),
    ("enums", &["id", "enum_type", "enum_key", "descriptions", "status", "created_at", "updated_at", "deleted_at"]),
    ("variables", &["id", "names", "type", "code", "variable", "remarks", "extension", "sort", "status", "created_at", "updated_at"]),
    ("standards", &["id", "names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "status", "created_at", "updated_at"]),
    ("shapes", &["id", "location", "names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "status", "created_at", "updated_at"]),
    ("formulas", &["id", "type", "shape_id", "names", "code", "columnar", "remarks", "extension", "sort", "status", "created_at", "updated_at"]),
    ("categories", &["id", "parent_id", "names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "standards", "shapes", "status", "created_at", "updated_at"]),
    ("products", &["id", "standard", "standard_id", "categories", "categories_id", "shapes", "shapes_id", "names", "grade", "code", "year", "covers", "svgs", "renders", "cads", "assemblies", "models", "detail", "parameters", "tolerance", "diameter_length", "drawing_limit", "formulas", "interpretation", "extension", "sort", "status", "created_at", "updated_at"]),
    ("interpretations", &["id", "product_id", "titles", "detail", "keywords", "extension", "status", "created_at", "updated_at"]),
    ("docs", &["id", "file_id", "title", "path", "detail", "credit", "times_expire", "extension", "status", "created_at", "updated_at"]),
    ("downloads", &["id", "doc_id", "uuid", "use_credit", "use_limit", "down_url", "times_expire", "created_at", "updated_at"]),
    ("modifications", &["key", "total", "created_at", "updated_at"]),
    ("docs_hits", &["id", "doc_id", "uuid", "created_at", "updated_at"]),
    ("docs_browsers", &["id", "doc_id", "uuid", "name", "product", "keywords", "extension", "created_at", "updated_at"]),
    ("docs_logs", &["id", "doc_id", "uuid", "name", "product", "keywords", "extension", "created_at", "updated_at"]),
    ("downloads_logs", &["id", "doc_id", "uuid", "name", "product", "keywords", "extension", "created_at", "updated_at"]),
    ("serials", &["id", "batch", "code", "secret", "type", "level", "days", "credit", "price", "remark", "extension", "times_expire", "uuid", "status", "created_at", "updated_at", "deleted_at"]),
];
