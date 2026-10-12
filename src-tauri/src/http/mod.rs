//! Rust 侧统一 HTTP 客户端。
//!
//! 迁移要求：**HTTP 请求统一在 Rust 侧发起，前端不直连后端**。
//! 前端不再持有 baseURL 与鉴权头，由 `http_request` 命令把请求转交到这里执行，
//! 并把状态码 / 响应头 / 响应体原样返回给前端（前端仅负责解析业务信封）。
//!
//! 与老实现的差异：老项目由渲染进程直接用 axios 访问后端（受 CORS / 证书 / 代理影响），
//! 新实现统一走原生 HTTP 栈（rustls，无系统 OpenSSL 依赖）。

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

/// HTTP 响应（已完整读入内存）
#[derive(Debug, Clone)]
pub struct HttpPayload {
    pub status: u16,
    pub headers: Map<String, Value>,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

impl HttpPayload {
    /// 尝试按 JSON 解析响应体；非 JSON 时返回 `{"__raw": "<文本>"}` 形式，避免前端拿到空数据。
    pub fn json(&self) -> Value {
        if self.body.is_empty() {
            return Value::Null;
        }
        match serde_json::from_slice::<Value>(&self.body) {
            Ok(value) => value,
            Err(_) => json!({ "__raw": self.text() }),
        }
    }

    /// 响应体文本（非法 UTF-8 用替换字符兜底）
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// 命令层返回结构（与前端约定：status / ok / headers / contentType / data）
    pub fn into_value(self) -> Value {
        let data = self.json();
        json!({
            "status": self.status,
            "ok": (200..300).contains(&self.status),
            "headers": Value::Object(self.headers),
            "contentType": self.content_type,
            "data": data,
        })
    }
}

/// HTTP 客户端（内部复用连接池）
#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "edtib-books/{}-tauri ({}/{})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH
            ))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// 发起请求。
    ///
    /// * `method`：GET / POST / PUT / PATCH / DELETE / HEAD（大小写不敏感）；
    /// * `headers`：JSON 对象，仅取字符串值；
    /// * `body`：JSON 值；字符串按原文发送（`application/x-www-form-urlencoded` 等场景），
    ///   其余类型按 `application/json` 序列化；`null` 表示无请求体。
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: Option<&Value>,
        body: Option<&Value>,
    ) -> AppResult<HttpPayload> {
        let url = normalize_url(url)?;
        let method = reqwest::Method::from_bytes(method.trim().to_uppercase().as_bytes())
            .map_err(|_| AppError::bad_request(format!("不支持的 HTTP 方法：{method}")))?;

        let mut req = self.client.request(method, url);

        if let Some(Value::Object(map)) = headers {
            for (name, value) in map {
                if let Some(text) = value.as_str() {
                    req = req.header(name, text);
                } else if !value.is_null() {
                    req = req.header(name, value.to_string());
                }
            }
        }

        if let Some(data) = body {
            if !data.is_null() {
                req = match data {
                    Value::String(text) => req.body(text.clone()),
                    other => req.json(other),
                };
            }
        }

        let response = req.send().await?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let mut header_map = Map::new();
        for (name, value) in response.headers().iter() {
            header_map.insert(
                name.to_string(),
                Value::String(value.to_str().unwrap_or_default().to_string()),
            );
        }

        let body = response.bytes().await?.to_vec();

        Ok(HttpPayload {
            status,
            headers: header_map,
            body,
            content_type,
        })
    }

    /// 下载到本地文件（`cache_file` 等场景），返回写入字节数
    pub async fn download_to(&self, url: &str, dest: &Path) -> AppResult<u64> {
        let url = normalize_url(url)?;
        // 注意：`url` 后续还要用于错误信息，这里用 `&str` 借用发起请求，避免被移动
        let response = self.client.get(url.as_str()).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(AppError::internal(format!(
                "下载失败：HTTP {} {}",
                status.as_u16(),
                url
            )));
        }
        let bytes = response.bytes().await?;
        if let Some(parent) = dest.parent() {
            crate::paths::ensure_dir(parent)?;
        }
        std::fs::write(dest, &bytes)?;
        Ok(bytes.len() as u64)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 仅允许 http / https 协议（避免前端传入 file:// 等本地协议造成越权读取），
/// 并拦截链路本地 / 云厂商元数据地址（SSRF 面收敛）。
/// 回环地址仍允许：开发态网关即为 127.0.0.1:9000。
pub fn normalize_url(url: &str) -> AppResult<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("请求地址不能为空"));
    }
    let lower = trimmed.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(AppError::bad_request(format!(
            "仅支持 http/https 请求地址：{trimmed}"
        )));
    }
    if let Some(host) = host_of(trimmed) {
        if is_blocked_host(&host) {
            return Err(AppError::bad_request(format!(
                "禁止访问的地址（链路本地 / 元数据服务）：{host}"
            )));
        }
    }
    Ok(trimmed.to_string())
}

/// 取 URL 的 host（去掉 scheme、端口与 userinfo）
fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, rest)| rest)?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let without_userinfo = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    // IPv6 字面量形如 [::1]:9000
    let host = if let Some(stripped) = without_userinfo.strip_prefix('[') {
        stripped.split(']').next().unwrap_or_default()
    } else {
        without_userinfo.split(':').next().unwrap_or(without_userinfo)
    };
    (!host.is_empty()).then(|| host.to_lowercase())
}

fn is_blocked_host(host: &str) -> bool {
    if matches!(
        host,
        "metadata" | "metadata.google.internal" | "instance-data" | "alibaba-intl-metadata"
    ) {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        // 169.254.0.0/16（含各云厂商 169.254.169.254）、阿里云 100.100.100.200、未指定地址
        return match ip {
            std::net::IpAddr::V4(v4) => {
                let [a, b, ..] = v4.octets();
                v4.is_unspecified() || (a == 169 && b == 254) || v4 == std::net::Ipv4Addr::new(100, 100, 100, 200)
            }
            // IPv6 链路本地前缀 fe80::/10（is_unicast_link_local 仍为 unstable，这里手工判定）
            std::net::IpAddr::V6(v6) => {
                v6.is_unspecified() || (v6.segments()[0] & 0xffc0) == 0xfe80
            }
        };
    }
    false
}

/// 判断字符串是否为远端地址（用于 `cache_file` 区分「下载 URL」与「本地文件路径」）
pub fn is_remote_url(value: &str) -> bool {
    let lower = value.trim().to_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// 从 URL 中提取文件名（去掉 query / fragment）
pub fn file_name_from_url(url: &str) -> Option<String> {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
}
