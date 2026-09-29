//! 数据表模型注册表（由临时脚本从老 Electron 项目 electron/src/model/*.ts 生成）。
//!
//! 生成来源：legacy_models.json（解析结果见实现报告「模型元数据来源」）
//! 说明：`fillable` / `guarded` / `hidden` / `casts` 与老模型 1:1 对应；
//!       `primary_key` 分别对应各表主键（id / key / path / uuid）。
//! 请勿手工编辑本文件——表结构变更请同步老模型后重新生成。

/// 字段转换类型（对应老项目 cast/*.ts）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    /// cast/integer.ts：parseInt，非法值取 0
    Integer,
    /// cast/integerOrNull.ts：空 / 非法值取 null
    IntegerOrNull,
    /// cast/string.ts：字符串化（false/null 原样保留）
    String,
    /// cast/json.ts：读取时 JSON.parse，写入时 JSON.stringify
    Json,
    /// cast/datetime.ts：格式化为 YYYY-MM-DD HH:mm:ss
    Datetime,
    /// cast/boolean.ts：读取 !!,  写入 '1' / '0'
    Boolean,
    /// cast/crypt.ts：AES-256-CBC（新项目未内置密钥，未配置时透传）
    Crypt,
}

/// 表模型定义
#[derive(Debug, Clone, Copy)]
pub struct ModelDef {
    /// 物理表名
    pub table: &'static str,
    /// 主键字段名
    pub primary_key: &'static str,
    /// 允许添加 / 更新的字段白名单
    pub fillable: &'static [&'static str],
    /// 不允许更新的字段黑名单
    pub guarded: &'static [&'static str],
    /// 输出时隐藏的字段
    pub hidden: &'static [&'static str],
    /// 字段转换规则
    pub casts: &'static [(&'static str, CastKind)],
    /// 老项目对应的 model 文件名（审计用）
    pub legacy: &'static str,
}

impl ModelDef {
    /// 字段转换类型（未声明的字段不转换）
    pub fn cast(&self, field: &str) -> Option<CastKind> {
        self.casts
            .iter()
            .find(|(name, _)| *name == field)
            .map(|(_, kind)| *kind)
    }

    /// 写库字段集合：主键 + fillable + guarded + hidden（与老实现 fields 数组一致）
    pub fn writable_fields(&self) -> Vec<&'static str> {
        let mut fields = vec![self.primary_key];
        for group in [self.fillable, self.guarded, self.hidden] {
            for field in group {
                if !fields.contains(field) {
                    fields.push(field);
                }
            }
        }
        fields
    }

    /// guarded + fillable（老实现用于判断是否需要 sort 排序）
    pub fn sortable_fields(&self) -> Vec<&'static str> {
        let mut fields: Vec<&'static str> = Vec::new();
        for group in [self.guarded, self.fillable] {
            for field in group {
                if !fields.contains(field) {
                    fields.push(field);
                }
            }
        }
        fields
    }

    /// 是否存在某字段（用于关键字检索字段裁剪）
    pub fn has_field(&self, field: &str) -> bool {
        self.writable_fields().contains(&field)
    }
}

/// 老项目允许访问的表（`loadModel` 的白名单）
pub static MODELS: &[ModelDef] = &[
    ModelDef {
        table: "categories",
        primary_key: "id",
        fillable: &["parent_id", "names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "standards", "shapes", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("parent_id", CastKind::IntegerOrNull), ("names", CastKind::Json), ("abbreviation", CastKind::String), ("covers", CastKind::Json), ("remarks", CastKind::Json), ("extension", CastKind::Json), ("total", CastKind::Integer), ("sort", CastKind::Integer), ("standards", CastKind::Json), ("shapes", CastKind::Json), ("status", CastKind::Integer)],
        legacy: "categories",
    },
    ModelDef {
        table: "configs",
        primary_key: "key",
        fillable: &["id", "key", "value"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("key", CastKind::String), ("value", CastKind::Crypt)],
        legacy: "configs",
    },
    ModelDef {
        table: "docs",
        primary_key: "id",
        fillable: &["names", "content", "type", "extension", "sort", "status"],
        guarded: &["created_at", "updated_at", "deleted_at"],
        hidden: &[],
        casts: &[("names", CastKind::Json), ("content", CastKind::String), ("type", CastKind::String), ("extension", CastKind::Json), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "docs",
    },
    ModelDef {
        table: "docs_browsers",
        primary_key: "id",
        fillable: &["doc_id", "uuid", "name", "product", "keywords", "extension"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("doc_id", CastKind::Integer), ("uuid", CastKind::String), ("name", CastKind::String), ("product", CastKind::String), ("keywords", CastKind::String), ("extension", CastKind::Json)],
        legacy: "docs_browsers",
    },
    ModelDef {
        table: "docs_hits",
        primary_key: "id",
        fillable: &["doc_id", "uuid"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("doc_id", CastKind::Integer), ("uuid", CastKind::String)],
        legacy: "docs_hits",
    },
    ModelDef {
        table: "docs_logs",
        primary_key: "id",
        fillable: &["doc_id", "uuid", "name", "product", "keywords", "extension"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("doc_id", CastKind::Integer), ("uuid", CastKind::String), ("name", CastKind::String), ("product", CastKind::String), ("keywords", CastKind::String), ("extension", CastKind::Json)],
        legacy: "docs_logs",
    },
    ModelDef {
        table: "downloads_logs",
        primary_key: "id",
        fillable: &["doc_id", "uuid", "name", "product", "keywords", "extension"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("doc_id", CastKind::Integer), ("uuid", CastKind::String), ("name", CastKind::String), ("product", CastKind::String), ("keywords", CastKind::String), ("extension", CastKind::Json)],
        legacy: "downloads_logs",
    },
    ModelDef {
        table: "enums",
        primary_key: "id",
        fillable: &["type", "key", "value", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("type", CastKind::String), ("key", CastKind::String), ("value", CastKind::String), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "enums",
    },
    ModelDef {
        table: "files",
        primary_key: "path",
        fillable: &["type", "url", "path", "source", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("type", CastKind::String), ("url", CastKind::String), ("path", CastKind::String), ("source", CastKind::String), ("status", CastKind::Integer)],
        legacy: "files",
    },
    ModelDef {
        table: "formulas",
        primary_key: "id",
        fillable: &["type", "shape_id", "names", "code", "columnar", "remarks", "extension", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("type", CastKind::Integer), ("shape_id", CastKind::IntegerOrNull), ("names", CastKind::Json), ("code", CastKind::String), ("columnar", CastKind::Crypt), ("remarks", CastKind::Json), ("extension", CastKind::Json), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "formulas",
    },
    ModelDef {
        table: "modifications",
        primary_key: "key",
        fillable: &["key", "total"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("key", CastKind::String), ("total", CastKind::Integer)],
        legacy: "modifications",
    },
    ModelDef {
        table: "products",
        primary_key: "id",
        fillable: &["standard", "standard_id", "categories", "categories_id", "shapes", "shapes_id", "names", "grade", "code", "year", "covers", "svgs", "renders", "cads", "assemblies", "models", "detail", "parameters", "tolerance", "diameter_length", "drawing_limit", "formulas", "interpretation", "extension", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("standard", CastKind::String), ("standard_id", CastKind::Integer), ("categories", CastKind::Crypt), ("categories_id", CastKind::Json), ("shapes", CastKind::Crypt), ("shapes_id", CastKind::Json), ("names", CastKind::Json), ("grade", CastKind::String), ("code", CastKind::String), ("year", CastKind::String), ("covers", CastKind::Json), ("svgs", CastKind::Crypt), ("renders", CastKind::Crypt), ("cads", CastKind::Crypt), ("assemblies", CastKind::Crypt), ("models", CastKind::Crypt), ("detail", CastKind::Crypt), ("parameters", CastKind::Crypt), ("tolerance", CastKind::Crypt), ("diameter_length", CastKind::Crypt), ("drawing_limit", CastKind::Crypt), ("formulas", CastKind::Crypt), ("interpretation", CastKind::Crypt), ("extension", CastKind::Json), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "products",
    },
    ModelDef {
        table: "schedules",
        primary_key: "id",
        fillable: &["name", "time", "handler", "status"],
        guarded: &["created_at", "updated_at", "deleted_at"],
        hidden: &[],
        casts: &[("name", CastKind::String), ("time", CastKind::String), ("handler", CastKind::Json), ("status", CastKind::Integer)],
        legacy: "schedules",
    },
    ModelDef {
        table: "secrets",
        primary_key: "id",
        fillable: &["title", "app_id", "app_secret", "app_iv", "app_url", "extension", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("title", CastKind::String), ("app_id", CastKind::String), ("app_secret", CastKind::Crypt), ("app_iv", CastKind::Crypt), ("app_url", CastKind::String), ("extension", CastKind::Json), ("status", CastKind::Integer)],
        legacy: "secrets",
    },
    ModelDef {
        table: "serials",
        primary_key: "id",
        fillable: &["batch", "code", "secret", "type", "level", "days", "credit", "price", "remark", "extension", "times_expire", "uuid", "status"],
        guarded: &["created_at", "updated_at", "deleted_at"],
        hidden: &["secret"],
        casts: &[("batch", CastKind::String), ("code", CastKind::String), ("secret", CastKind::String), ("type", CastKind::Integer), ("level", CastKind::Integer), ("days", CastKind::Integer), ("credit", CastKind::Integer), ("price", CastKind::Integer), ("remark", CastKind::String), ("extension", CastKind::Json), ("times_expire", CastKind::Datetime), ("uuid", CastKind::String), ("status", CastKind::Integer)],
        legacy: "serials",
    },
    ModelDef {
        table: "shapes",
        primary_key: "id",
        fillable: &["location", "names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("location", CastKind::Integer), ("names", CastKind::Json), ("abbreviation", CastKind::String), ("covers", CastKind::Json), ("remarks", CastKind::Json), ("extension", CastKind::Json), ("total", CastKind::Integer), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "shapes",
    },
    ModelDef {
        table: "standards",
        primary_key: "id",
        fillable: &["names", "abbreviation", "covers", "remarks", "extension", "total", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("names", CastKind::Json), ("abbreviation", CastKind::String), ("covers", CastKind::Json), ("remarks", CastKind::Json), ("extension", CastKind::Json), ("total", CastKind::Integer), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "standards",
    },
    ModelDef {
        table: "users",
        primary_key: "uuid",
        fillable: &["id", "nickname", "truename", "email", "mobile", "avatar", "auth_token", "uuid", "times_expire", "credit", "extension", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("nickname", CastKind::String), ("truename", CastKind::String), ("email", CastKind::String), ("mobile", CastKind::String), ("avatar", CastKind::String), ("password", CastKind::String), ("salt", CastKind::String), ("auth_token", CastKind::String), ("uuid", CastKind::String), ("secret", CastKind::String), ("times_expire", CastKind::Datetime), ("credit", CastKind::Integer), ("extension", CastKind::Json), ("status", CastKind::Integer)],
        legacy: "users",
    },
    ModelDef {
        table: "variables",
        primary_key: "id",
        fillable: &["names", "type", "code", "variable", "remarks", "sort", "status"],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("names", CastKind::Json), ("type", CastKind::String), ("code", CastKind::String), ("variable", CastKind::String), ("remarks", CastKind::Json), ("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "variables",
    },
    ModelDef {
        // 新项目新增：产品筛选缓存表（前端 ProductsFilter.vue 直接调用 db_find_all('products_filter')，
        // 老白名单中没有该表；此处按宽松模式（fillable 为空 => 写入时不做字段裁剪）放行）
        table: "products_filter",
        primary_key: "id",
        fillable: &[],
        guarded: &["created_at", "updated_at"],
        hidden: &[],
        casts: &[("sort", CastKind::Integer), ("status", CastKind::Integer)],
        legacy: "-",
    },
];

/// 额外放行的表（不属于老 MODELS 白名单，但前端桥接层需要）
pub static EXTRA_TABLES: &[&str] = &["products_filter"];

/// 按表名查找模型定义
pub fn lookup(table: &str) -> Option<&'static ModelDef> {
    MODELS.iter().find(|model| model.table == table)
}

/// 表名是否允许访问
pub fn is_allowed(table: &str) -> bool {
    lookup(table).is_some() || EXTRA_TABLES.contains(&table)
}

/// 所有允许访问的表名
pub fn allowed_tables() -> Vec<&'static str> {
    let mut tables: Vec<&'static str> = MODELS.iter().map(|model| model.table).collect();
    for table in EXTRA_TABLES {
        if !tables.contains(table) {
            tables.push(table);
        }
    }
    tables
}
