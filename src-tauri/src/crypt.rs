//! 字段级加密（CastCrypt）与哈希工具。
//!
//! 迁移说明（重要）：
//!   老 Electron 侧 `cast/crypt.ts` 使用固定内置 key/iv 的 AES-256-CBC（新版格式在密文 hex 前
//!   拼接随机 IV）。按照迁移要求，新项目**不保留**老 Electron 的 VITE_APP_SECRET / app_secret
//!   密钥链，因此本模块**不内置任何默认密钥**：
//!     * 配置了 `EDTIB_FIELD_CRYPT_SECRET`（可配 `EDTIB_FIELD_CRYPT_IV`）时，按 AES-256-CBC 加解密；
//!     * 未配置时字段级加解密自动「透传」（读写原样），保证数据不因缺少密钥而被破坏。
//!   → 若需要读取老数据中的加密字段（如 products.categories / configs.value），
//!     需由业务提供密钥并以环境变量注入；否则这些字段以密文原样返回（见实现报告「遗留事项」）。

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use serde_json::Value;
use sha2::{Digest, Sha256};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// 字段加密器（无密钥时为透传模式）
#[derive(Clone)]
pub struct FieldCrypt {
    key: Option<[u8; 32]>,
    iv: Option<[u8; 16]>,
    /// 未显式配置 `EDTIB_FIELD_CRYPT_IV` 时按随机 IV 前置写出；
    /// 显式配置 IV 表示处于「与老 Electron 数据互读写」的兼容模式，沿用固定 IV。
    random_iv: bool,
}

impl std::fmt::Debug for FieldCrypt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldCrypt")
            .field("enabled", &self.enabled())
            .finish()
    }
}

impl Default for FieldCrypt {
    fn default() -> Self {
        Self::from_env()
    }
}

impl FieldCrypt {
    /// 从环境变量读取密钥（缺失即透传模式）
    pub fn from_env() -> Self {
        let secret = std::env::var("EDTIB_FIELD_CRYPT_SECRET")
            .ok()
            .filter(|v| !v.trim().is_empty());
        let iv = std::env::var("EDTIB_FIELD_CRYPT_IV")
            .ok()
            .filter(|v| !v.trim().is_empty());
        Self::from_secret(secret.as_deref(), iv.as_deref())
    }

    /// 由业务密钥派生 AES-256-CBC 的 key/iv（SHA-256 派生，避免明文长度不足）
    pub fn from_secret(secret: Option<&str>, iv: Option<&str>) -> Self {
        match secret {
            Some(secret) => {
                let mut hasher = Sha256::new();
                hasher.update(secret.as_bytes());
                let key: [u8; 32] = hasher.finalize().into();

                let iv_source = iv.unwrap_or(secret);
                let mut iv_hasher = Sha256::new();
                iv_hasher.update(iv_source.as_bytes());
                let digest = iv_hasher.finalize();
                let mut iv_bytes = [0u8; 16];
                iv_bytes.copy_from_slice(&digest[..16]);

                Self {
                    iv: Some(iv_bytes),
                    key: Some(key),
                    // 未显式给定 IV => 写出时改用随机 IV 前置（不再使用 secret 派生的固定 IV）
                    random_iv: iv.is_none(),
                }
            }
            None => Self {
                iv: None,
                key: None,
                random_iv: true,
            },
        }
    }

    /// 是否启用加解密（未配置密钥时为 false，走透传）
    pub fn enabled(&self) -> bool {
        self.key.is_some() && self.iv.is_some()
    }

    /// 解密：hex 密文 -> JSON 值。
    ///
    /// 兼容老格式：密文 hex 长度满足 `(len - 32) % 32 == 0` 且总长 > 64 时，
    /// 前 32 个 hex 字符视为随机 IV（与老实现 `decrypt` 的前置 IV 约定一致）。
    pub fn decrypt_value(&self, cipher_hex: &str) -> Option<Value> {
        let (key, default_iv) = match (self.key, self.iv) {
            (Some(key), Some(iv)) => (key, iv),
            _ => return None,
        };
        let raw = hex::decode(cipher_hex.trim()).ok()?;
        if raw.is_empty() {
            return None;
        }
        // 尝试两种 IV：内置默认 IV / 前置 IV
        let candidates: Vec<([u8; 16], Vec<u8>)> = {
            let mut list = vec![(default_iv, raw.clone())];
            if raw.len() > 16 && (raw.len() - 16) % 16 == 0 {
                let mut iv = [0u8; 16];
                iv.copy_from_slice(&raw[..16]);
                list.push((iv, raw[16..].to_vec()));
            }
            list
        };
        for (iv, payload) in candidates {
            let mut buffer = payload.clone();
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

    /// 加密：JSON 值 -> hex 密文。
    ///
    /// [安全加固] 每次加密随机生成 IV 并前置到密文（`hex(iv || cipher)`），
    /// 避免固定 IV 下「同明文 => 同密文」的 CBC 模式泄露。
    /// 解密侧 `decrypt_value` 本就兼容固定 IV 与前置 IV 两种格式，存量数据仍可读。
    pub fn encrypt_value(&self, value: &Value) -> Option<String> {
        let (key, fixed_iv) = match (self.key, self.iv) {
            (Some(key), Some(iv)) => (key, iv),
            _ => return None,
        };
        let text = serde_json::to_string(value).ok()?;
        let iv: [u8; 16] = if self.random_iv {
            let mut random = [0u8; 16];
            rand::thread_rng().fill(&mut random);
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

/// SHA-256（hex）
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
