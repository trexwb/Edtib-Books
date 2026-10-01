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
        match keychain_probe() {
            KeychainEntry::Found(hex) => {
                return Ok(DbKey {
                    hex: hex.to_lowercase(),
                    source: KeySource::Keychain,
                })
            }
            // [数据安全] Keychain 中已有记录但内容非法：绝不带 -U 覆盖重写，
            // 否则既有 SQLCipher 库将永久无法解密。先尝试同目录密钥文件（迁移遗留），
            // 仍不可用则显式报错，交由人工处理。
            KeychainEntry::Corrupted(raw) => {
                let detail = if raw.is_empty() { "（空值）".to_string() } else { "（长度或字符集不符合 64 位十六进制）".to_string() };
                eprintln!("[keychain] Keychain 记录非法{detail}，service={KEYCHAIN_SERVICE}, account={KEYCHAIN_ACCOUNT}");
                if let Some(hex) = key_file_read_existing(data_dir) {
                    return Ok(DbKey {
                        hex,
                        source: KeySource::KeyFile,
                    });
                }
                return Err(AppError::internal(format!(
                    "Keychain 中的数据库密钥记录非法{detail}，且未找到可用的密钥文件；为避免覆盖后既有加密库永久无法解密，已停止初始化，请人工确认后处理"
                )));
            }
            KeychainEntry::Absent => {}
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
        // [数据安全] 文件存在但内容非法时不覆盖重写，避免既有加密库永久失读
        return Err(AppError::internal(
            "密钥文件 .sqlcipher-key 已存在但内容非法（需 64 位十六进制字符）；为避免覆盖后既有加密库无法解密，已停止初始化，请人工确认后处理",
        ));
    }
    crate::paths::ensure_dir(data_dir)?;
    std::fs::write(&file, &fresh)?;
    restrict_permissions(&file);
    Ok(DbKey {
        hex: fresh,
        source: KeySource::Generated,
    })
}

/// 只读取既有密钥文件（内容合法才返回），不创建、不覆盖
#[cfg(target_os = "macos")]
fn key_file_read_existing(data_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(data_dir.join(KEY_FILE_NAME)).ok()?;
    let content = content.trim().to_string();
    is_valid_hex_key(&content).then(|| content.to_lowercase())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

/// Keychain 记录状态（区分「不存在」与「存在但非法」，后者不可覆盖）
#[cfg(target_os = "macos")]
enum KeychainEntry {
    Absent,
    Found(String),
    Corrupted(String),
}

#[cfg(target_os = "macos")]
fn keychain_probe() -> KeychainEntry {
    let output = match Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
            "-w",
        ])
        .output()
    {
        Ok(output) => output,
        Err(_) => return KeychainEntry::Absent,
    };
    if !output.status.success() {
        // security 以非 0 退出码表示「无该记录」
        return KeychainEntry::Absent;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return KeychainEntry::Corrupted(String::new());
    }
    if is_valid_hex_key(&value) {
        KeychainEntry::Found(value)
    } else {
        KeychainEntry::Corrupted(value)
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
