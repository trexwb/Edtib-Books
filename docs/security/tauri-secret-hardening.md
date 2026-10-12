# Tauri v2 桌面应用生产密钥（TAURI_APP_ID/SECRET/IV）保密方案

> 适用范围：Tauri + Vite 桌面端，前端框架 __ / Tauri 版本 __ / Rust 本地 http 实现 __ / 密钥算法 __ 以占位。
> 三条链路：`npm run dev`（.env.development 直连测试 API）｜`npm run tauri:dev`（.env.test 经 Tauri 本地 http）｜`npm run build`（.env.production 经 Tauri 本地 http）。
> 硬约束：① 编入前端 bundle 或 Rust 二进制的常量一律视为已泄露，须给出攻击者成本与提取手法；② 每结论配命令/代码；③ 禁编造名，v1/v2 差异显式标注，不确定写【需确认：…】；④ 覆盖前端与 Rust 两侧。

## 0 实测口径（2026-10-12 以代码为准，此前的假设已在括号内更正）
1. **Tauri 版本**：v2（`src-tauri/Cargo.toml:23` `tauri = { version = "2" }`），未使用 v1 兼容层。
2. **Rust 本地 http 实现**：**应用内 `reqwest` 直连**，不是 `tauri-plugin-http`——出网只发生在
   `src-tauri/src/proxy/request.rs`（console `reqwest` 0.13 + 显式 `rustls` 0.23/ring，books 0.12
   `rustls-tls`）。渲染层没有 `http` 插件权限，也不允许直接出网。
   【残留：两仓 reqwest 主版本不同，属框架同构待对齐项】
3. **密钥算法**：**AES-256-CBC（PKCS7 填充）**，不是 AES-256-GCM——`aes = "0.8"` + `cbc = "0.1"`，
   新写入为随机 IV 前置（`hex(iv‖cipher)`），固定 IV 仅用于读老数据（见 `crypt.rs`）。
   签名侧为 `md5(sha256(appId+ts+nonce)+appSecret)+ts`（本地链路与出站链路同一族算法）。
   因此本文 §3「IV 复用」缺陷的修正是 CBC 语境下的随机 IV，而非换 AEAD；CBC 无完整性校验，
   密文可被位翻转攻击，这一残留风险在 §9 的方案决策里作为「不能只靠加密」的依据之一。

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

### 8.4 加固轮落地（2026-10-12，console / books 双侧同构）

本轮把「凭证不可绝对保密」这一前提转成**可验证的边界**：出网目标、请求头、本地文件、
跨环境四个方向收口，并把检查做成可重复执行的命令。

| 方向 | 落地 | 位置 |
| --- | --- | --- |
| 出网目标 | 配置地址优先、表内 `app_url` 仅兜底且必须同源；正式包只允许 `ALLOWED_GATEWAY_ORIGINS`；`path` 拼接禁止 authority 变形；跨主机重定向不跟随 | `proxy/request.rs::pin_origin/resolve_origin/join_target/client` |
| 请求头 | 转发前剔除 `cookie/x-forwarded-*/x-real-ip/client-ip/sec-*/proxy-*/origin/referer/host` 等可伪造或可被网关当作决策输入的头；`authorization`（用户令牌）与业务头保留 | `proxy/request.rs::is_stripped_header` |
| 本地文件 | `fs_read_file/fs_write_file/fs_delete_file` 一律拒绝 `.env`/`.db`/`.db-wal`/`.db-shm`/`.sqlcipher-key`（含父目录名判定），此前 `fs_read_file(<AppData>/console.env)` 可直接读出两套密钥 | `commands/fs.rs::ensure_not_protected` |
| 跨环境 | 缺出站地址明确失败（删除内置兜底）；开发/测试构建默认拒绝正式网关（`EDTIB_ALLOW_PROD_EGRESS=1` 才放行）；`tauri:dev` 默认拒读 `.env.production`；本地链路密钥缺失/长度非法/与 `VITE_APP_*` 不一致时注入脚本 exit 1 | `config.rs` / `proxy/request.rs` / `scripts/with-tauri-env.mjs` |
| 重放 | 本地链路时间戳增加上界（未来 60 s 之外拒绝），不再只有下界 | `proxy/middleware.rs::CLOCK_SKEW_SECS` |
| 产物校验 | `npm run scan:dist`（Web 资源）+ `npm run scan:installer`（打包目录/二进制），命中 `TAURI_APP_*` 或非本环境链路密钥即 exit 1；已串到 `vite:build` / `build` 链尾 | `scripts/scan-*.mjs` + `package.json` |

实测结论（本轮取证，命令与退出码为准）：
- `npm run scan:dist`：console 265 文件 / books 160 文件，`TAURI_APP_*` 命中 **0**，
  仅本地链路密钥按设计存在；`scan:installer` 对已展开的 `.app` 明文产物命中 **0**。
- 扫描器自检：把 `.env.test` 的 `TAURI_APP_SECRET` 临时注入一份 `dist` 副本后重扫，
  正确报 `FAIL 出站网关凭证` 并 exit 1（证明 0 命中不是"没在扫"）。
- 已知盲区（未判定为安全）：`.dmg` 是压缩容器，明文扫描不穿透；Tauri 内嵌前端资源随二进制
  分发，抽出的是压缩字节流——要证明"取不出"需挂载/解包后重扫，或按 §2 做运行时内存提取。
  这两项在本轮**未执行**，因此只声称"未发现明文泄漏"，不声称"不可提取"。
- 仓库侧：`.env* / console.env / books.env` 均在 `.gitignore`，`git ls-files` 未跟踪任何环境文件；
  `books/.env.production` 与 `console/.env.production` 的 `TAURI_APP_ID/SECRET/IV` 已清空，
  正式出站凭证不再以任何形式存在于工作副本里，只存在于发布机部署的密钥文件或 `secrets` 表。

---

## 9 差距清单与凭证方案决策（2026-10-12，结论均以代码为准）

### 9.1 逐条对照目标行为的差距

| # | 目标行为 | 现状 | 证据 |
| --- | --- | --- | --- |
| 1 | `npm run dev` 只吃 `.env.development`、直连测试 API、不碰 Tauri | 达标：`vite --mode development`；`isTauri()` 为假 ⇒ 保留 axios 默认 adapter，`/api` 经 vite 代理到 `gateway-dev` | `package.json` dev 脚本；`src/utils/request.ts:285`；`vite.config.ts:79-86` |
| 2 | `npm run tauri:dev` 渲染层用 `.env.test` 的 `VITE_APP_*`，业务只走 Tauri | 达标：`beforeDevCommand = npm run dev:test`；Tauri 宿主下 adapter 换成 `invoke('proxy_request')`，`/api` 仅作路径前缀 | `tauri.conf.json:10`；`src/utils/tauriAdapter.ts:270` |
| 3 | Rust 从「独立安全来源」读 `TAURI_APP_*` | 达标：进程环境变量 → `<AppData>/{console,books}.env`，源码无兜底；`with-tauri-env.mjs` 只注入 `TAURI_*/EDTIB_*/NODE_ENV` | `src-tauri/src/config.rs:138+`；`scripts/with-tauri-env.mjs` |
| 4 | 测试态不得意外拿到正式凭证 / 打到正式网关 | 达标（本轮收口）：`.env.production` 的 `TAURI_APP_*` 已清空；dev 构建的出站地址是 dev 组（指纹与 `.env.development` 的 `VITE_APP_*` 同组）；`resolve_origin` 在 development 且未 `EDTIB_ALLOW_PROD_EGRESS=1` 时拒绝 `gateway.edtib.com`；`tauri:dev` 拒读 `.env.production` | `proxy/request.rs::resolve_origin`；`config.rs::PRODUCTION_GATEWAY_HOSTS` |
| 5 | `npm run build` = 正式 Tauri 构建 + 凭证注入 | **口径不符，需注意**：`npm run build` = `vite:build`（Web 产物 + `scan:dist`）→ `tauri:build` → `make-dmg` → `scan:installer`。**全链不做任何凭证注入**——正式出站凭证只在运行期来自发布机的 `<AppData>/*.env` 或本地 `secrets` 表。CI 仅注入 updater 签名私钥 | `package.json` build/vite:build/tauri:build；`scripts/with-updater-key.mjs` |
| 6 | 正式凭证不进 Web 产物 | 达标且有机器证据：`scan:dist` 对 265/160 个产物文件命中 0；注入已知密钥自检可 FAIL + exit 1 | `scripts/scan-dist-secret.mjs` |
| 7 | 正式凭证不「无保护地写入客户端可读取的配置文件」 | **不达标（架构限制，非本仓可修）**：`<AppData>/console.env` 是明文 `KEY=VALUE`，任何本机可读文件者即得网关凭证；`db/seed.rs:77` 还会把它复制进本地 `secrets` 表（`app_secret/app_iv` 走 `Cast::Crypt`，未配置 `TAURI_FIELD_CRYPT_SECRET` 时明文透传，且默认密钥由 `TAURI_LINK_SECRET` 派生——同一份材料已在 bundle 里） | `config.rs::env_file_path`；`db/seed.rs:63-77`；`models/mod.rs:124`；`config.rs:153` |
| 8 | 三种模式无静默跨环境回退 | 达标：缺出站地址明确报错；链路密钥缺失/长度非法/与 `VITE_APP_*` 不一致时注入脚本 exit 1；环境文件缺失时两个扫描器 exit 1；dev 打正式网关需显式开关 | `resolve_origin`；`with-tauri-env.mjs`；`scan-*.mjs:83-87` |

### 9.2 方案对比（服务端能力已实测，非假设）

**方案 A 服务端代理——服务端已具备该形态，但对本项目是"换一层"而非"新增能力"。**
网关已有多个「客户端只做业务、上游密钥只在服务端」的先例：
`FastenerAiService.chat` 用 `process.env.AI_LLM_API_KEY` 代 `customerId` 调
`/chat/completions`（`src/app/Services/Fastener/FastenerAiService.ts:243-246`，调用前有会话归属
`:213` 与额度预检 `:217-223`、计费 `:336-343`）；短信（`resources/sms/AliyunSmsClient.ts:52-65`，
密钥来自 DB 配置行 `SmsConfigService.ts:46`）、OSS（`Attachments/OssService.ts:38-45`，
下发预签名 URL `FastenerToolService.ts:509-511`）、微信/支付宝
（`resources/payments/WechatOAuthService.ts:84-95`）同属一类。
**限制**：桌面端的"第三方 API"就是网关本身，所以 A 在这里退化为「客户端不再以应用身份签名」，
真正的收益要给的是 AI/短信/OSS 这类真第三方——那部分已经是 A。**A 不适用于 `proxy_request` 主链路。**

**方案 B 短期凭证——服务端地基齐全，缺的是"设备身份"这一块。**
已具备：不透明 64 位随机令牌（`AccountsUsersService.ts:301-303`，非 JWT）、TTL
`token_time=1800` / `front_token_time=86400`（`ConsoleAuthorizeController.ts:51`、
`Controllers/Front/AuthorizeController.ts:53`）、半程滑动刷新 `X-New-Token`
（`Middleware/RefreshToken.ts:27-37`）、轮换即撤销（`AccountsUsersService.ts:294-307` +
`/signOut` `/logout`，`routes/apis/console/authorize.ts:24,27`）、Redis NX 一次性 nonce
（`AuthenticateSecret.ts:71-76`）、带 `expires` 的 HMAC 短时签名 URL（`routes/front.ts:37`）。
缺口（实测确认）：① 全仓无设备/安装维度概念——迁移、模型、`src/types/entities` 里
`device|machine_id|fingerprint|client_id|install_id` **零命中**，只有 `clients_logs` 记一份
`client` JSON 审计块（无查询路径）；② `secrets` 表的 `times_expire` 与 `permissions`
（`database/migrations/20260121054100_secrets.ts:18-24`）**只写不校验**——
`AuthenticateSecret` 只判 `status`（`:56-59`），过期与授权范围不生效；③ 要让签名密钥改为
"服务端按会话签发"，必须动 `md5(sha256(appId+ts+nonce)+appSecret)` 这条协议（客户端与
`AuthenticateSecret.ts:65-67` 同时改），属跨仓协议变更。**结论：B 可行且是唯一根治方向，
但需要后端排期，不能由客户端仓库单方面完成。**

**方案 C 客户端持有应用凭证——当前实际形态，明确不对本机用户保密。**
可达路径清单（本机即可，无需越权）：读 `<AppData>/console.env` 明文；读未加密的本地
`secrets` 表；进程内存中的 `OnceLock<String>`（退出零化缺失）；调试器/Frida 钩 `env_var`。
`strip = true`、AES-CBC 落库、改名 `TAURI_APP_*` 都只提高成本，不构成保密
（CBC 还无完整性，可位翻转）。**只有在接受该风险并配齐权限限制、配额、监控、轮换、泄露响应
时才可使用**，而这五项里"权限限制/过期"目前是空列（见 B 缺口②）。

### 9.3 决策与实施顺序

1. **当下（已落地）**：维持 C 的协议不变，把暴露面从"处处可读"收到"两处"——
   环境文件 + 本地 secrets 表；出网目标/请求头/本地文件/跨环境四个方向做成机器校验（§8.4）。
2. **短期收口（不改协议，客户端可独立完成，需负责人确认）**：
   发布机把 `console.env/books.env` 权限设为 `0600` 并纳入部署脚本；console 本地库补齐
   SQLCipher 或至少强制 `TAURI_FIELD_CRYPT_SECRET` 独立配置（不复用 `TAURI_LINK_SECRET`）；
   `secrets` 表 `app_url/app_secret` 从渲染层可读的管理 CRUD 中摘出（读路径只给 Rust）。
3. **根治（需后端，推荐 B）**：设备注册表 + 首启鉴权换取短时签名密钥（内存驻留、不落盘、
   可撤销），同时把 `secrets.times_expire/permissions` 真正接入 `AuthenticateSecret`，
   使"泄露一把密钥"的影响面从"全租户长期可用"降为"单设备单时间窗"。
4. **明确不承诺**：任何混淆、加密、Rust 编译或环境变量隔离都不能阻止本机凭证提取（§2、§8.3）。

### 9.4 阻碍项（需要外部信息才能继续）
- 【需确认：`console.env/books.env` 如何到达用户机】`tauri.conf.json` 未配 `bundle.resources`，
  CI 只注入签名私钥；若依赖人工分发，`0600` 与轮换都无强制点。
- 【需后端：B 的设备鉴权端点与密钥签发协议】涉及 `AuthenticateSecret`/`VerifySignature`
  双改，客户端仓库无权单方面变更签名算法（本轮已按约束保持协议原样）。
- 【同构待对齐】`reqwest` console 0.13 / books 0.12、console 明文 rusqlite / books SQLCipher、
  插件集差异——出网栈不同会让"同构"结论只能覆盖逻辑层而非依赖层。



