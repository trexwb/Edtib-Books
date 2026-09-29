//! Tauri 命令层：与前端桥接层 `src/bridge/channels.ts` 的命令契约一一对应。
//!
//! | 命令空间 | 命令 | 对应 |
//! |---|---|---|
//! | db | db_find_all / db_get_list / db_find_one / db_create / db_update / db_delete / db_bulk_create | `BRIDGE_CHANNELS.db.*` |
//! | fs | fs_read_file / fs_write_file / fs_delete_file / fs_list_files / fs_get_user_data_path / fs_get_documents_path / fs_exists | `BRIDGE_CHANNELS.fs.*` |
//! | system | system_get_info | `BRIDGE_CHANNELS.system.getInfo` |
//! | app | get_app_version / check_update / confirm_update / restart_app / cache_file | `BRIDGE_CHANNELS.app.*` |
//! | http | http_request / http_download | 迁移要求：HTTP 统一在 Rust 侧发起（前端桥接层未声明，属扩展能力） |
//! | 诊断 | bootstrap_report | 启动自检（非桥接契约，便于排查） |

pub mod app;
pub mod db;
pub mod fs;
pub mod http;
pub mod system;
