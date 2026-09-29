//! SQLCipher 数据库密钥管理。
//!
//! 需求：
//!   * 密钥每设备随机生成、持久化保存，首次启动时初始化；
//!   * macOS 上优先使用系统 Keychain 保存（`security` 命令行，Keychain 服务名 com.edtib.books.sqlcipher）；
//!   * 其他平台降级为数据目录下的权限受限密钥文件（0600）。
//!
//! 注意：本模块只负责「本地数据加密密钥」，与老 Electron 的
//! VITE_APP_SECRET / app_secret（业务接口签名密钥链）无关，后者不迁移。

use std::path::Path;
use std::process::Command;

use rand::RngCore;

use crate::error::{AppError, AppResult};

#[cfg(target_os = "macos")]
const KEYCHAIN_SERVICE: &str = "com.edtib.books.sqlcipher";
#[cfg(target_os = "macos")]
const KEYCHAIN_ACCOUNT: &str = "database-key";

/// 密钥文件（非 macOS 平台的降级存储）
const KEY_FILE_NAME: &str = ".sqlcipher-key";

/// 密钥来源（用于启动自检报告）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// 来自环境变量 EDTIB_DB_KEY_HEX（调试 / 自动化场景）
    EnvVar,
    /// 来自 macOS Keychain
    Keychain,
    /// 来自数据目录密钥文件
    KeyFile,
    /// 本次随机生成
    Generated,
}

impl KeySource {
    pub fn as_str(self) -> &'static str {
        match self {
            KeySource::EnvVar => "env:EDTIB_DB_KEY_HEX",
            KeySource::Keychain => "macos-keychain",
            KeySource::KeyFile => "key-file",
            KeySource::Generated => "generated",
        }
    }
}

/// 数据库密钥（hex 编码的 32 字节）
#[derive(Debug, Clone)]
pub struct DbKey {
    pub hex: String,
    pub source: KeySource,
}

/// 读取已有密钥；不存在则随机生成并持久化（首次启动初始化）。
pub fn load_or_create(data_dir: &Path) -> AppResult<DbKey> {
    // 1) 环境变量优先（自动化测试 / CI）
    if let Ok(hex_key) = std::env::var("EDTIB_DB_KEY_HEX") {
        let hex_key = hex_key.trim().to_string();
        if is_valid_hex_key(&hex_key) {
            return Ok(DbKey {
                hex: hex_key,
                source: KeySource::EnvVar,
            });
        }
        return Err(AppError::bad_request(
            "环境变量 EDTIB_DB_KEY_HEX 非法（需 64 位十六进制字符）",
        ));
    }

    // 2) macOS Keychain
    #[cfg(target_os = "macos")]
    {
        if let Some(hex) = keychain_read() {
            if is_valid_hex_key(&hex) {
                return Ok(DbKey {
                    hex: hex.to_lowercase(),
                    source: KeySource::Keychain,
                });
            }
        }
        let generated = generate_hex();
        match keychain_write(&generated) {
            Ok(()) => {
                return Ok(DbKey {
                    hex: generated,
                    source: KeySource::Generated,
                })
            }
            Err(err) => {
                // Keychain 不可用时降级为密钥文件，保证应用仍可启动
                eprintln!("[keychain] 写入 Keychain 失败，降级为密钥文件: {err}");
            }
        }
        return key_file_load_or_create(data_dir, generated);
    }

    // 3) 其他平台：密钥文件
    #[cfg(not(target_os = "macos"))]
    {
        key_file_load_or_create(data_dir, generate_hex())
    }
}

/// 生成 32 字节随机密钥（hex 64 字符）
pub fn generate_hex() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn is_valid_hex_key(hex: &str) -> bool {
    hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn key_file_load_or_create(data_dir: &Path, fresh: String) -> AppResult<DbKey> {
    let file = data_dir.join(KEY_FILE_NAME);
    if let Ok(content) = std::fs::read_to_string(&file) {
        let content = content.trim().to_string();
        if is_valid_hex_key(&content) {
            return Ok(DbKey {
                hex: content.to_lowercase(),
                source: KeySource::KeyFile,
            });
        }
    }
    crate::paths::ensure_dir(data_dir)?;
    std::fs::write(&file, &fresh)?;
    restrict_permissions(&file);
    Ok(DbKey {
        hex: fresh,
        source: KeySource::Generated,
    })
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

#[cfg(target_os = "macos")]
fn keychain_read() -> Option<String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
            "-w",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "macos")]
fn keychain_write(hex_key: &str) -> Result<(), String> {
    // -U：存在则更新，避免重复添加报错
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
            "-w",
            hex_key,
            "-U",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
