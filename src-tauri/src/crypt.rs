//! 加密工具：字段级加密（CastCrypt）与哈希工具。
//!
//! [框架统一] 基元层与 `console/src-tauri/src/crypt.rs` 同款口径
//! （等价老项目 `electron/src/config/cryptTool.ts` / Node `crypto`）：
//!   * `md5` / `sha256`：摘要（hex）；
//!   * `encrypt` / `decrypt`：aes-256-cbc + hex，key/iv 为 utf8 原始字符串；
//!   * `encrypt_iv_prefixed` / `decrypt_adaptive`：随机 IV 前置格式 `iv(hex):密文(hex)`；
//!   * `encrypt_default` / `decrypt_default`：使用本地库字段级密钥（[`crate::config`] 读取）。
//! 与 console 的实现差异仅两处：随机 IV 使用既有依赖 `rand::OsRng`（不新增 getrandom），
//! 日志走 `eprintln!`（books 未引入 log crate）。
//!
//! 迁移说明（重要）：
//!   books 口径下**不保留** VITE_APP_SECRET / app_secret 本地链路密钥链
//!   （网关凭据在前端、Rust 透传，不参与签名/加密），本模块**不内置任何默认密钥**：
//!     * 配置 `TAURI_FIELD_CRYPT_SECRET` 时启用字段加密；未配置时读写透传，数据不被破坏；
//!     * 长度为 32 的 secret 走 console 同构模式（utf8 原始密钥 + `iv:密文` 随机 IV 写出）；
//!     * 其他长度沿用 books 老派生模式（key=sha256(secret)，`hex(iv||cipher)` 前置 IV）；
//!     * 读取侧两种格式互备兼容（按是否含 `:` 分流，失败再试另一格式），存量数据始终可读。

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use rand::{rngs::OsRng, RngCore};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::config;

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// [框架统一] 等价 `cryptTool.md5`（books 无本地链路签名需求，保留与 console 同构）
#[allow(dead_code)]
pub fn md5(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

/// 等价 `cryptTool.sha256`。
pub fn sha256(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// 等价 `cryptTool.encrypt`：先 `JSON.stringify`，再 aes-256-cbc 加密为 hex。
/// 失败时返回空串（对应老项目返回 `false` 后由 `CastCrypt.set` 转成 `''`）。
pub fn encrypt(value: &Value, key: &str, iv: &str) -> String {
    let plain = match serde_json::to_string(value) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("[crypt] 加密前置序列化失败: {err}");
            return String::new();
        }
    };

    let cipher = match Aes256CbcEnc::new_from_slices(key.as_bytes(), iv.as_bytes()) {
        Ok(cipher) => cipher,
        Err(err) => {
            eprintln!("[crypt] aes-256-cbc 初始化失败: {err}");
            return String::new();
        }
    };

    let mut buffer = plain.into_bytes();
    let message_len = buffer.len();
    buffer.resize(message_len + 16, 0);
    match cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len) {
        Ok(ciphertext) => hex::encode(ciphertext),
        Err(err) => {
            eprintln!("[crypt] aes-256-cbc 加密失败: {err}");
            String::new()
        }
    }
}

/// 等价 `cryptTool.decrypt`：hex → aes-256-cbc 解密 → `JSON.parse`。
/// 失败返回 `None`（对应老项目返回 `false`）。
/// `iv` 为**原始 16 字节**（ASCII 字符串形态，等价 Node `Buffer.from(iv, 'utf8')`）。
pub fn decrypt(ciphertext: &str, key: &str, iv: &str) -> Option<Value> {
    decrypt_with_iv(ciphertext, key.as_bytes(), iv.as_bytes())
}

/// aes-256-cbc 解密：`iv` 为已解码的 16 字节。
fn decrypt_with_iv(ciphertext: &str, key: &[u8], iv: &[u8]) -> Option<Value> {
    let mut data = hex::decode(ciphertext).ok()?;
    let cipher = Aes256CbcDec::new_from_slices(key, iv).ok()?;
    let plain = cipher.decrypt_padded_mut::<Pkcs7>(&mut data).ok()?;
    let text = std::str::from_utf8(plain).ok()?;
    serde_json::from_str(text).ok()
}

/// 使用本地库字段级密钥加密（等价 `CastCrypt.set` 的调用方式）。
#[allow(dead_code)] // console 同构入口；books 数据层统一经 FieldCrypt 进出
pub fn encrypt_default(value: &Value) -> String {
    encrypt(value, config::field_crypt_secret(), config::field_crypt_iv())
}

/// 使用本地库字段级密钥解密（等价 `CastCrypt.get` 的调用方式）。
#[allow(dead_code)] // 同上
pub fn decrypt_default(ciphertext: &str) -> Option<Value> {
    decrypt(ciphertext, config::field_crypt_secret(), config::field_crypt_iv())
}

/// 等价 web 前端 `encrypt()`（`src/utils/request.ts`）：
/// 每次生成随机 16 字节 IV，返回 `iv(hex) + ':' + 密文(hex)`；失败返回 `None`。
pub fn encrypt_iv_prefixed(value: &Value, key: &str) -> Option<String> {
    if key.len() != 32 {
        eprintln!("[crypt] 加密密钥长度必须为 32（当前 {}）", key.len());
        return None;
    }

    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);
    let iv_hex = hex::encode(iv);

    let plain = serde_json::to_string(value).ok()?;
    let cipher = Aes256CbcEnc::new_from_slices(key.as_bytes(), &iv).ok()?;
    let mut buffer = plain.into_bytes();
    let message_len = buffer.len();
    buffer.resize(message_len + 16, 0);
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
        .ok()?;

    Some(format!("{iv_hex}:{}", hex::encode(ciphertext)))
}

/// 自适应解密：优先按 `iv(hex) + ':' + 密文(hex)`（web 前端约定，随机 IV）解析；
/// 无 IV 前缀时退回固定 IV（老 electron 约定）。
pub fn decrypt_adaptive(ciphertext: &str, key: &str, legacy_iv: &str) -> Option<Value> {
    match split_iv_prefixed(ciphertext) {
        // 随机 IV 约定：前缀是 IV 的 hex 文本，必须解码成 16 字节再解密
        // （等价 Node `Buffer.from(ivHex, 'hex')`）；直接当 ASCII 传入会得到 32 字节而解密失败。
        Some((iv_hex, payload)) => {
            let iv = hex::decode(iv_hex).ok()?;
            decrypt_with_iv(payload, key.as_bytes(), &iv)
        }
        None => decrypt(ciphertext, key, legacy_iv),
    }
}

/// 拆出 `iv(hex):payload`；IV 必须是 16 字节（32 位 hex）。
fn split_iv_prefixed(ciphertext: &str) -> Option<(&str, &str)> {
    let (iv_hex, payload) = ciphertext.split_once(':')?;
    if iv_hex.len() != 32 || !iv_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some((iv_hex, payload))
}

/// 字段加密器（未配置密钥为透传模式；双格式兼容读写）
#[derive(Clone)]
pub struct FieldCrypt {
    /// console 同构模式：utf8 原始密钥（恰为 32 字符时启用）
    node_key: Option<String>,
    /// console 同构模式的固定 IV（16 字符；缺省则固定 IV 兼容解密不可用）
    node_iv: Option<String>,
    /// books 老派生模式：key = sha256(secret)
    legacy_key: Option<[u8; 32]>,
    /// books 老派生模式：iv = sha256(iv 或 secret)[..16]
    legacy_iv: Option<[u8; 16]>,
    /// 老派生模式下未显式配置 IV => 写出随机 IV 前置（`hex(iv||cipher)`）
    random_iv: bool,
}

impl std::fmt::Debug for FieldCrypt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldCrypt")
            .field("enabled", &self.enabled())
            .field("mode", if self.node_key.is_some() { &"node" } else { &"derived" })
            .finish()
    }
}

impl Default for FieldCrypt {
    fn default() -> Self {
        Self::from_env()
    }
}

impl FieldCrypt {
    /// 从统一配置入口读取密钥（进程环境变量 > 密钥文件 books.env；缺失即透传模式）
    pub fn from_env() -> Self {
        let secret = Some(config::field_crypt_secret()).filter(|v| !v.is_empty());
        let iv = Some(config::field_crypt_iv()).filter(|v| !v.is_empty());
        Self::from_secret(secret.as_deref(), iv.as_deref())
    }

    /// 由业务密钥构造加密器：
    ///   * secret 长度 32 => console 同构模式（原始 utf8 密钥，`iv(hex):密文(hex)` 写出）；
    ///   * 其他长度 => books 老派生模式（sha256 派生，`hex(iv||cipher)` 写出）；
    ///   * 两种模式的解密路径始终互相兼容，只按密文形态分流。
    pub fn from_secret(secret: Option<&str>, iv: Option<&str>) -> Self {
        let Some(secret) = secret else {
            return Self {
                node_key: None,
                node_iv: None,
                legacy_key: None,
                legacy_iv: None,
                random_iv: true,
            };
        };

        let node_key = (secret.len() == 32).then(|| secret.to_string());
        let node_iv = iv.filter(|v| v.len() == 16).map(|v| v.to_string());

        let key: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
        let iv_source = iv.unwrap_or(secret);
        let digest = Sha256::digest(iv_source.as_bytes());
        let mut iv_bytes = [0u8; 16];
        iv_bytes.copy_from_slice(&digest[..16]);

        Self {
            node_key,
            node_iv,
            legacy_key: Some(key),
            legacy_iv: Some(iv_bytes),
            // 未显式给定 IV => 派生模式写出改用随机 IV 前置（不再使用 secret 派生的固定 IV）
            random_iv: iv.is_none(),
        }
    }

    /// 是否启用加解密（未配置密钥时为 false，走透传）
    pub fn enabled(&self) -> bool {
        self.legacy_key.is_some()
    }

    /// 解密：密文 -> JSON 值。
    ///
    /// 按形态分流：含 `:` 前缀走 console 同构格式解密；纯 hex 走 books 老格式
    /// （派生密钥 + 固定 IV / 前置 IV 两种尝试）；一侧失败再试另一侧。
    pub fn decrypt_value(&self, cipher_text: &str) -> Option<Value> {
        let text = cipher_text.trim();
        let node = || {
            let key = self.node_key.as_deref()?;
            decrypt_adaptive(text, key, self.node_iv.as_deref().unwrap_or(key))
        };
        let legacy = || self.decrypt_legacy(text);
        if text.contains(':') {
            node().or_else(legacy)
        } else {
            legacy().or_else(node)
        }
    }

    /// books 老格式解密：hex 密文，派生密钥；兼容固定 IV 与前置 IV 两种写法。
    ///
    /// 兼容老格式：密文字节满足 `len > 16 && (len - 16) % 16 == 0` 时，
    /// 前 16 字节视为随机 IV（与老实现 `decrypt` 的前置 IV 约定一致）。
    fn decrypt_legacy(&self, cipher_hex: &str) -> Option<Value> {
        let (key, default_iv) = match (self.legacy_key, self.legacy_iv) {
            (Some(key), Some(iv)) => (key, iv),
            _ => return None,
        };
        let raw = hex::decode(cipher_hex).ok()?;
        if raw.is_empty() {
            return None;
        }
        // 尝试两种 IV：内置默认 IV / 前置 IV
        let mut candidates: Vec<([u8; 16], Vec<u8>)> = vec![(default_iv, raw.clone())];
        if raw.len() > 16 && (raw.len() - 16) % 16 == 0 {
            let mut iv = [0u8; 16];
            iv.copy_from_slice(&raw[..16]);
            candidates.push((iv, raw[16..].to_vec()));
        }
        for (iv, mut buffer) in candidates {
            if let Ok(plain) = Aes256CbcDec::new(&key.into(), &iv.into()).decrypt_padded_mut::<Pkcs7>(&mut buffer) {
                if let Ok(text) = std::str::from_utf8(plain) {
                    if let Ok(value) = serde_json::from_str::<Value>(text) {
                        return Some(value);
                    }
                    return Some(Value::String(text.to_string()));
                }
            }
        }
        None
    }

    /// 加密：JSON 值 -> 密文。
    ///
    /// [安全加固] 两种模式都随机 IV 前置：
    ///   * console 同构模式（32 字符密钥）：`iv(hex):密文(hex)`；
    ///   * 派生模式：`hex(iv||cipher)`；
    /// 解密侧 `decrypt_value` 双格式兼容，存量数据仍可读。
    pub fn encrypt_value(&self, value: &Value) -> Option<String> {
        if let Some(key) = &self.node_key {
            return encrypt_iv_prefixed(value, key);
        }
        let (key, fixed_iv) = match (self.legacy_key, self.legacy_iv) {
            (Some(key), Some(iv)) => (key, iv),
            _ => return None,
        };
        let text = serde_json::to_string(value).ok()?;
        let iv: [u8; 16] = if self.random_iv {
            let mut random = [0u8; 16];
            OsRng.fill_bytes(&mut random);
            random
        } else {
            fixed_iv
        };
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        // PKCS7 填充：预留一个块（16 字节）的填充空间，用 encrypt_padded_mut 原地加密
        let mut buffer = text.as_bytes().to_vec();
        let message_len = buffer.len();
        buffer.resize(message_len + 16, 0);
        let encrypted = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .ok()?;
        let output = if self.random_iv {
            let mut prefixed = iv.to_vec();
            prefixed.extend_from_slice(encrypted);
            prefixed
        } else {
            encrypted.to_vec()
        };
        Some(hex::encode(output))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// 测试专用合成密钥（非任何环境真实凭证；密钥一律来自环境变量/密钥文件）
    const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";
    const TEST_IV: &str = "fedcba9876543210";

    #[test]
    fn digest_length() {
        assert_eq!(md5("edtib").len(), 32);
        assert_eq!(sha256("edtib").len(), 64);
    }

    #[test]
    fn disabled_field_crypt_passes_through() {
        let crypt = FieldCrypt::from_secret(None, None);
        assert!(!crypt.enabled());
        assert_eq!(crypt.encrypt_value(&json!({"a": 1})), None);
        assert_eq!(crypt.decrypt_value("whatever"), None);
    }

    #[test]
    fn node_mode_roundtrip_uses_colon_format() {
        let crypt = FieldCrypt::from_secret(Some(TEST_KEY), Some(TEST_IV));
        let value = json!({"hello": "网关", "list": [1, 2, 3]});
        let encrypted = crypt.encrypt_value(&value).expect("加密成功");
        assert!(encrypted.contains(':'));
        assert_eq!(crypt.decrypt_value(&encrypted), Some(value));
    }

    #[test]
    fn derived_mode_roundtrip_still_readable() {
        // 非 32 字符密钥：沿用 books 老派生模式（hex(iv||cipher)，无冒号）
        let crypt = FieldCrypt::from_secret(Some("short-secret"), None);
        let value = json!({"categories": ["m6", "m8"]});
        let encrypted = crypt.encrypt_value(&value).expect("加密成功");
        assert!(!encrypted.contains(':'));
        assert_eq!(crypt.decrypt_value(&encrypted), Some(value));
    }

    #[test]
    fn node_mode_reads_legacy_electron_fixed_iv_hex() {
        // 老 electron 口径：utf8 固定 IV、无冒号前缀的 hex 密文 —— 存量数据必须可读
        let value = json!({"legacy": true});
        let encrypted = encrypt(&value, TEST_KEY, TEST_IV);
        let crypt = FieldCrypt::from_secret(Some(TEST_KEY), Some(TEST_IV));
        assert_eq!(crypt.decrypt_value(&encrypted), Some(value));
    }

    #[test]
    fn node_mode_reads_colonless_random_iv_via_adaptive_iv_equal_key() {
        // 无 16 字符 IV 配置时，含冒号密文走随机 IV 前缀分支
        let crypt = FieldCrypt::from_secret(Some(TEST_KEY), None);
        let value = json!("0123456789abcdefghijklmnopqrstuv");
        let encrypted = crypt.encrypt_value(&value).expect("加密成功");
        assert_eq!(crypt.decrypt_value(&encrypted), Some(value));
    }
}
