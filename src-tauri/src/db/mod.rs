//! 数据层（本地 SQLite / SQLCipher）。
//!
//! 模块划分：
//!   * [`connection`]：SQLCipher 连接、密钥 PRAGMA、建表迁移、旧库搬迁；
//!   * [`schema`]：由老项目 knex 迁移文件转译而来的 SQLite DDL（22 张表）；
//!   * [`query`]：filters、order 到 SQL 的构造；
//!   * [`row`]：行读取转换（表元数据与字段 cast 见 [`crate::models`]）；
//!   * [`execute`]：查询 / 计数 / 写入 / 更新 / 删除。

pub mod connection;
pub mod execute;
pub mod query;
pub mod row;
pub mod schema;

pub use connection::Database;
