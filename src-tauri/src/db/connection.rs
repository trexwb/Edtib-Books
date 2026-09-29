//! SQLCipher 加密数据库的连接与初始化。
//!
//! 与老 Electron 实现的差异（本次迁移的既定要求）：
//!   * 老实现使用 `knex + sqlite3`，PRAGMA key 被注释未启用（明文库）；
//!   * 新实现固定使用 SQLCipher 加密（`rusqlite` 的 `bundled-sqlcipher-vendored-openssl`），
//!     密钥来自系统 Keychain（见 [`crate::keychain`]），每设备随机生成并持久化；
//!   * 首次启动时若存在老版 Electron 的明文数据库（`<legacy userData>/run_*.bin`），
//!     通过 `sqlcipher_export` 搬迁为加密库（仅当新库尚未创建时执行）。

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, OptionalExtension};

use super::schema;
use crate::error::{AppError, AppResult};
use crate::keychain::DbKey;

/// SQLCipher 加密数据库句柄（内部串行化访问：SQLite 单写多读，命令层并发调用下同样安全）
pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
    cipher_version: Option<String>,
    key_source: &'static str,
}

impl Database {
    /// 打开数据库并设置 SQLCipher 密钥。
    ///
    /// 密钥为本设备随机生成的 32 字节原始密钥（hex），使用 SQLCipher 原始密钥语法
    /// `PRAGMA key = "x'<hex>'"` 传入，跳过口令 KDF，避免老式 passphrase 带来的兼容问题。
    pub fn open(path: &Path, key: &DbKey) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            crate::paths::ensure_dir(parent)?;
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        conn.busy_timeout(Duration::from_secs(10))?;

        // 探测当前链接的 SQLite 是否带 SQLCipher codec（bundled-sqlcipher 应始终可用）
        let cipher_version: Option<String> = conn
            .query_row("PRAGMA cipher_version;", [], |row| row.get::<_, String>(0))
            .optional()
            .unwrap_or(None);

        if cipher_version.is_some() {
            conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key.hex))?;
            // 密钥校验：密钥错误时访问 sqlite_master 会立刻失败（文件不是 DB 或密钥不匹配）
            conn.query_row("SELECT count(*) FROM sqlite_master;", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| {
                AppError::internal(format!(
                    "数据库密钥校验失败（密钥不匹配或文件损坏）：{err}"
                ))
            })?;
        } else {
            eprintln!(
                "[db] 警告：当前链接的 SQLite 未启用 SQLCipher codec，数据库将以**明文**写入 {}",
                path.display()
            );
        }

        // 常规 PRAGMA（加密设置完成后才能执行）
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA cache_size = -8000;",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
            cipher_version,
            key_source: key.source.as_str(),
        })
    }

    /// 数据库文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `PRAGMA cipher_version` 结果；`None` 表示未启用 SQLCipher
    pub fn cipher_version(&self) -> Option<&str> {
        self.cipher_version.as_deref()
    }

    /// 密钥来源（env / macos-keychain / key-file / generated）
    pub fn key_source(&self) -> &'static str {
        self.key_source
    }

    /// 是否处于加密状态
    pub fn encrypted(&self) -> bool {
        self.cipher_version.is_some()
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::internal("数据库连接锁获取失败（可能存在未捕获的 panic）"))
    }

    /// 在锁保护下访问底层连接
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.lock()?;
        f(&conn)
    }

    /// 建表迁移（幂等）：执行 [`schema::SCHEMA_SQL`] 并校验目标表是否齐全。
    pub fn migrate(&self) -> AppResult<Vec<String>> {
        self.with(|conn| {
            conn.execute_batch(schema::SCHEMA_SQL)?;
            conn.execute_batch("PRAGMA user_version = 1;")?;

            let mut missing: Vec<String> = Vec::new();
            for table in schema::TABLES {
                let exists: Option<String> = conn
                    .query_row(
                        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1;",
                        [table],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?;
                if exists.is_none() {
                    missing.push((*table).to_string());
                }
            }
            if !missing.is_empty() {
                return Err(AppError::internal(format!(
                    "数据库建表失败，缺失表：{}",
                    missing.join(", ")
                )));
            }
            Ok(missing)
        })
    }

    /// 已存在的业务表名（排除 sqlite 内部表）
    pub fn table_names(&self) -> AppResult<Vec<String>> {
        self.with(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name;",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            let mut names = Vec::new();
            for row in rows {
                names.push(row?);
            }
            Ok(names)
        })
    }

    /// 单表行数（缺失表返回 0）
    pub fn table_row_count(&self, table: &str) -> AppResult<i64> {
        let ident = super::query::quote_ident(table)?;
        self.with(|conn| {
            let count: Option<i64> = conn
                .query_row(&format!("SELECT count(*) FROM {ident};"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .optional()
                .unwrap_or(None);
            Ok(count.unwrap_or(0))
        })
    }
}

/// 把老版 Electron 的**明文** SQLite 库导出为 SQLCipher 加密库（首次启动搬迁旧数据）。
///
/// 仅在 `target` 尚不存在时调用；任何失败都由调用方回退为「使用全新加密库」。
pub fn import_plain_sqlite(target: &Path, legacy: &Path, key_hex: &str) -> AppResult<()> {
    if let Some(parent) = target.parent() {
        crate::paths::ensure_dir(parent)?;
    }
    let conn = Connection::open(target)?;
    let cipher_version: Option<String> = conn
        .query_row("PRAGMA cipher_version;", [], |row| row.get::<_, String>(0))
        .optional()
        .unwrap_or(None);
    if cipher_version.is_none() {
        return Err(AppError::unavailable(
            "当前链接未启用 SQLCipher，无法搬迁旧数据库",
        ));
    }

    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key_hex))?;
    conn.execute(
        "ATTACH DATABASE ?1 AS legacy KEY '';",
        [legacy.to_string_lossy().to_string()],
    )?;
    conn.execute_batch("SELECT sqlcipher_export('main', 'legacy');")?;
    conn.execute_batch("DETACH DATABASE legacy;")?;
    Ok(())
}
