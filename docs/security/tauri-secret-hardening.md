# Tauri v2 桌面应用生产密钥（TAURI_APP_ID/SECRET/IV）保密方案

> 适用范围：Tauri + Vite 桌面端，前端框架 __ / Tauri 版本 __ / Rust 本地 http 实现 __ / 密钥算法 __ 以占位。
> 三条链路：`npm run dev`（.env.development 直连测试 API）｜`npm run tauri:dev`（.env.test 经 Tauri 本地 http）｜`npm run build`（.env.production 经 Tauri 本地 http）。
> 硬约束：① 编入前端 bundle 或 Rust 二进制的常量一律视为已泄露，须给出攻击者成本与提取手法；② 每结论配命令/代码；③ 禁编造名，v1/v2 差异显式标注，不确定写【需确认：…】；④ 覆盖前端与 Rust 两侧。

## 0 待确认项（CONTEXT 中 __ 缺失）
1. **Tauri 版本**：确认为 v2？以下按 v2 写，v1 差异在文中显式标注（配置/权限/插件 API 不同）。
2. **Rust 本地 http 实现**：`reqwest` 直连？还是 `tauri-plugin-http`？以下按 Rust command 内 `reqwest` 假设，【需确认：是否改用 tauri-plugin-http】。
3. **密钥算法**：AES-256-GCM / AES-CBC / SM4 / ChaCha20？以下按 AES-256-GCM（AEAD）假设，【需确认：实际算法与分组/IV 长度】。

---

## 1 一句话结论
任何写死进前端 bundle 或 Rust 二进制的 `TAURI_APP_SECRET/IV` 都可被提取（成本 5 分钟至数小时，见 §2），唯一根治是 L0——密钥不下发、改由服务端代理或运行时设备鉴权下发短期密钥且仅驻留内存；L1/L2 只做拖延与可检测，不是保密保证。

---

## 2 威胁建模表
| 资产 | 攻击者能力 | 提取路径 | 成本 | 风险 |
|---|---|---|---|---|
| TAURI_APP_SECRET | 拿到安装包/解包 | 前端产物 `strings`/`grep`/DevTools | ~5min | S |
| TAURI_APP_SECRET | 同左 | Rust 二进制 `strings`/`objdump`/`Ghidra` | ~30min–2h | S |
| SECRET/IV（运行时） | 本机运行+调试 | Frida 钩 `getenv`/解密例程读内存 | ~1–4h | S |
| 请求签名/明文 | 本机 MITM | `mitmproxy` 抓 loopback，无 TLS pin 即明文 | ~10min | A |
| CI 密钥 | 仓库/CI 读权限 | `gh run view --log` 若 echo 了密钥 | ~min | S |

风险等级：S=可提取且影响生产；A=条件可提取；B=高成本；C=理论。

---

## 3 设计缺陷审查（固定 IV + 共享 secret 对称加密）
- **IV 复用**：同 key+同 IV 加密相同明文得相同密文，退化为 ECB 级泄露；攻击者用已知明文(crib)拖出其它密文。修正：每消息 `rand::random::<[u8;12]>()`，nonce 前缀拼接密文。
- **密钥等价泄露**：单一共享 secret 全网等价，一处提取全设备沦陷，无设备隔离。修正：每设备密钥由设备 attest 派生，或服务端按设备发短期 key。
- **无前向安全**：历史流量用旧 key 加密，轮换不救已捕获流量；密钥轮换后旧密文仍可解。修正：AEAD + 服务端短时 token，密钥不下发。
- **secret 进 bundle/binary**：见 §2，必泄露。修正：Rust 运行时 `std::env::var` 读取，绝不 `const`。

---

## 4 分层方案
**L0 根治（强度 S｜成本 高｜触发：可改后端）**
服务端代理：桌面端只持 `app_id`，敏感调用转服务端，服务端带 secret 调生产 API；或运行时设备鉴权下发短期 key 仅驻留内存（Drop 即 `zeroize`）。彻底消除客户端持长期 secret。

**L1 缓解（强度 A｜成本 中｜触发：必须本地持密钥）**
app_id 与 secret 拆分（id 可公开）；secret 经白盒/混淆+完整性校验；开反调试（`std::env::var` 检测 debugger）；Rust release `strip=true`；前端不内联 `VITE_TAURI_*`。

**L2 兜底（强度 B｜成本 低｜触发：任何情况）**
CI secrets 注入不进 git；`gitleaks` 产物密钥扫描；密钥轮换+吊销；代码签名(notarization/signtool)；日志脱敏（禁打 secret）。

---

## 5 落地代码（分文件标路径）

### 5.1 `.env.development`（web dev，直连测试 API；仅 VITE_ 给前端）
```bash
VITE_API_BASE=https://test-api.__
# 生产 TAURI_* 绝不以 VITE_ 暴露（Vite 仅内联 VITE_ 前缀 → 会进 bundle）
```

### 5.2 `.env.test`（tauri:dev，经 Rust 本地 http）
```bash
TAURI_APP_ID=test_app_id
TAURI_APP_SECRET=test_only_secret
TAURI_APP_IV=test_iv_16bytes_
API_BASE=https://test-api.__
```

### 5.3 `.env.production`（仅 CI secret 注入，不落 git）
```bash
TAURI_APP_ID=prod_app_id
TAURI_APP_SECRET=__   # 来自 CI secret，禁止提交
TAURI_APP_IV=__
API_BASE=https://api.__
```

### 5.4 `.gitignore`（books/ 根，追加）
```gitignore
.env.production
.env.local
# .env.development / .env.test 仅含非生产值可入库；生产 secret 永不可入库
```

### 5.5 `src-tauri/tauri.conf.json`（v2，无任何 secret）
```json
{
  "productName": "__",
  "version": "1.0.0",
  "identifier": "__",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:__",
    "beforeBuildCommand": "TAURI_BUILD=1 npm run vite:build && node scripts/remove-dist-sw.mjs",
    "frontendDist": "../dist"
  },
  "app": { "windows": [{ "title": "__", "width": 1024, "height": 768 }], "security": { "csp": null } },
  "bundle": { "active": true, "targets": "all" },
  "plugins": {}
}
```
> v1 差异：【需确认】v1 用 `tauri.security` + `tauri.allowlist.http`，无 `app`/`plugins` 顶层结构，权限写在 allowlist。

### 5.6 `src-tauri/capabilities/default.json`（v2 权限，http 走 Rust command 不需前端 http 权限）
```json
{ "identifier": "default", "description": "default", "windows": ["main"], "permissions": ["core:default"] }
```
> v1 差异：v1 在 `tauri.conf.json` 的 `allowlist` 配 `http`，无 capabilities 文件。

### 5.7 `src-tauri/src/secure.rs`（运行时读取 + 内存保护，禁 const）
```rust
use std::env;
use zeroize::Zeroize;            // 【需确认：引入 zeroize 依赖】
pub struct AppSecrets { pub app_id: String, pub app_secret: String, pub iv: [u8; 16] }
impl Drop for AppSecrets {
    fn drop(&mut self) { self.app_id.zeroize(); self.app_secret.zeroize(); }
}
pub fn load_secrets() -> Result<AppSecrets, Box<dyn std::error::Error>> {
    let app_id = env::var("TAURI_APP_ID")?;
    let app_secret = env::var("TAURI_APP_SECRET")?;   // 运行时读，绝不编译进二进制
    let iv: [u8;16] = env::var("TAURI_APP_IV")?.as_bytes().try_into().map_err(|_| "IV!=16")?;
    Ok(AppSecrets { app_id, app_secret, iv })
}
```

### 5.8 `src-tauri/src/secure.rs`（修正固定 IV：随机每消息 nonce，AES-GCM）
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};   // 【需确认：算法为 AES-256-GCM】
use rand::random;
pub fn encrypt(secret: &[u8], pt: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(secret));
    let nonce = random::<[u8; 12]>();                 // 随机 IV，杜绝复用
    let mut ct = cipher.encrypt(Nonce::from_slice(&nonce), pt).unwrap();
    let mut out = nonce.to_vec(); out.extend(ct); out // nonce 前缀随密文下发
}
```

### 5.9 `src-tauri/src/lib.rs`（Rust 本地 http command，secret 不外发）
```rust
#[tauri::command]
async fn proxy_request(path: String, body: serde_json::Value) -> Result<String, String> {
    let sec = load_secrets().map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();              // 【需确认：Rust http 用 reqwest】
    let url = format!("{}{}", env::var("API_BASE").unwrap(), path);
    let resp = client.post(&url)
        .header("X-App-Id", &sec.app_id)              // id 可公开
        .header("X-App-Sig", sign(&sec, &body))       // 用 secret 签名，secret 不出进程
        .json(&body).send().await.map_err(|e| e.to_string())?;
    Ok(resp.text().await.unwrap())
}
```

### 5.10 前端调用改造（books/src，禁内联 secret）
```ts
// ❌ 禁止：import.meta.env.VITE_TAURI_APP_SECRET 会被内联进单文件 bundle
// ✅ 经 Tauri command，secret 仅在 Rust 侧
import { invoke } from '@tauri-apps/api/core'   // v1: '@tauri-apps/api'
const res = await invoke('proxy_request', { path: '/orders', body: {...} })
```

### 5.11 GitHub Actions（`.github/workflows/release.yml`，运行时注入不编译）
```yaml
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - name: Inject prod secrets at runtime
        env:
          TAURI_APP_ID: ${{ secrets.TAURI_APP_ID }}
          TAURI_APP_SECRET: ${{ secrets.TAURI_APP_SECRET }}
          TAURI_APP_IV: ${{ secrets.TAURI_APP_IV }}
        run: npm run tauri:build   # Rust 运行时 std::env::var 读取 → 不进二进制
```

### 5.12 GitLab CI（`.gitlab-ci.yml`）
```yaml
build_desktop:
  variables:
    TAURI_APP_ID: "$TAURI_APP_ID_PROD"      # protected variable
  script:
    - export TAURI_APP_SECRET="$TAURI_APP_SECRET_PROD"
    - export TAURI_APP_IV="$TAURI_APP_IV_PROD"
    - npm run tauri:build
```

---

## 6 验证命令 + checklist
**提取验证（证明是否泄露）**
```bash
# 前端单文件产物
grep -rao 'TAURI_APP_SECRET[^"'"'"' ]*' dist/ && echo "LEAK:bundle" || echo "clean:bundle"
strings dist/index.html | grep -i 'app_secret\|TAURI_APP'
# Rust 二进制（release 默认 strip）
strings target/release/__ | grep -i 'TAURI_APP_SECRET'
objdump -s -j .rodata target/release/__ | grep -i secret
# 解包验证（macOS）
tar tzf target/release/bundle/macos/*.app.tar.gz | head
# 运行时内存（Frida）
frida-trace -i "*getenv*" -i "*encrypt*" -f target/release/__
```

**Acceptance checklist**
- [ ] `grep -rn 'TAURI_APP_SECRET' src-tauri/src books/src` 无字面量常量
- [ ] `strings target/release/__` 无 `TAURI_APP_SECRET` 明文
- [ ] `tauri.conf.json` 不含任何 secret 字段
- [ ] `npm run build` 后 `dist/index.html` 无 `VITE_TAURI_*` 内联
- [ ] CI 日志 `gh run view --log` 无 secret 回显
- [ ] `Cargo.toml [profile.release] strip = true` 已启用
- [ ] 已配置代码签名（macOS notarization / Windows signtool）
- [ ] 密钥轮换 + 吊销流程文档化

---

## 7 残留风险 + 演进
**做不到的**：任何到达客户端的长期 secret，运行时必可被 Frida/debugger 从内存提取（§2 成本 1–4h）；root/jailbreak 可绕过反调试与 TLS pin；白盒混淆仅提高成本不保密。故 L0 是唯一保密保证。

**30 天**：CI 接 `gitleaks` 扫描 + 产物密钥扫描；密钥轮换与吊销；启用代码签名；日志脱敏；`strip=true`。

**90 天**：落地 L0——服务端代理（桌面只持 `app_id`）或设备鉴权下发短时 token 仅驻留内存，彻底移除客户端长期 secret；前端改走 `@tauri-apps/api/core` invoke 无密钥调用。

---

## 8 实测落地结论（edtib books / console，2026-10-12 执行）

### 8.1 已做对（经代码核查，无需改动）
- **密钥不走编译**：`src-tauri/src/config.rs` 经 `std::env::var` + 密钥文件 `<AppData>/books.env`（console 为 `console.env`）运行时读取，源码无内置默认密钥；`tauri.conf.json` 无 secret 字段。
- **新数据随机 IV**：`crypt.rs` 的 `encrypt_iv_prefixed` 用 `getrandom`/`OsRng` 每消息随机 IV，固定 IV 仅用于解密老格式存量（文档 §3 缺陷在新链路已规避）。
- **`strip = true` 已存在**于两份 `[profile.release]`；`.env.production` 原已在 `.gitignore`。
- **本地链路密钥 ≠ 网关凭证**：bundle 内联的 `VITE_APP_SECRET` 是本地链路对（webview↔Rust），与出站网关 `TAURI_APP_SECRET` 是不同密钥，网关凭证不在前端 bundle。

### 8.2 本次落地（L2 兜底，已改文件）
- `books/.gitignore` + `console/.gitignore`：新增 `books.env` / `console.env`（生成的含密钥文件，无点前缀不被 `.env*` 覆盖，原会误提交）。
- 新增 `books/.github/workflows/secret-scan.yml` + `console/.github/workflows/secret-scan.yml`：`gitleaks/gitleaks-action@v2` 在 push/PR 全量扫描。
- console 既有 `release.yml` 仅注入 updater 签名私钥（`TAURI_SIGNING_PRIVATE_KEY`），与文档 §5.11 一致，未改动。

### 8.3 待决 / 残留（需用户决策或后端）
- **L0 缺口（风险 S）**：生产网关凭证 `TAURI_APP_*` 存于 `<AppData>/books.env` 随分发到达每台用户机，任何可读该文件者即得网关凭证（文档 §2 二进制/文件提取路径）。根治 = L0 服务端代理或设备鉴权下发短时 token。
- **【需确认：生产 `books.env`/`console.env` 分发机制】**：`tauri.conf.json` `bundle` 未配 `resources`，CI 仅注入签名私钥，该文件如何落到用户 AppData 不在本仓库，需确认（首启下载 / 独立配置步骤 / 手动分发）。
- **`VITE_APP_SECRET` 内联进生产 bundle**：本地链路按设计需前端持有，但属文档 §2 前端泄漏向量；建议后续改由 Rust 运行时 `invoke` 下发，实现 L0-lite。
- **内存零化**：secret 存 `OnceLock<String>` 静态，进程退出不零化，Frida 内存提取仍可行（§2）；`OnceLock` 不可变，零化需重构存储，低优先待决。

