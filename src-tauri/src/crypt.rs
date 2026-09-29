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
use serde_json::Value;
use sha2::{Digest, Sha256};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// 字段加密器（无密钥时为透传模式）
#[derive(Clone)]
pub struct FieldCrypt {
    key: Option<[u8; 32]>,
    iv: Option<[u8; 16]>,
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
                    key: Some(key),
                    iv: Some(iv_bytes),
                }
            }
            None => Self {
                key: None,
                iv: None,
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

    /// 加密：JSON 值 -> hex 密文（与老实现 `JSON.stringify` + AES-256-CBC 等价）
    pub fn encrypt_value(&self, value: &Value) -> Option<String> {
        let (key, iv) = match (self.key, self.iv) {
            (Some(key), Some(iv)) => (key, iv),
            _ => return None,
        };
        let text = serde_json::to_string(value).ok()?;
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        // PKCS7 填充：预留一个块（16 字节）的填充空间，用 encrypt_padded_mut 原地加密
        let mut buffer = text.as_bytes().to_vec();
        let message_len = buffer.len();
        buffer.resize(message_len + 16, 0);
        let encrypted = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .ok()?;
        Some(hex::encode(encrypted))
    }
}

/// SHA-256（hex）
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
