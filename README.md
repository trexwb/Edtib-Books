# 紧固助手（个人版）— 桌面客户端

edtib books 桌面端由 **Electron 迁移至 Tauri v2**。业务代码已完成迁入：渲染层 `src/`（60 个 `views/*.vue`、router / store / api / utils / i18n / 桥接层）与 Rust 侧 `src-tauri/`（命令层、SQLite、加解密、请求转发、密钥链、定时任务、在线更新）。

- 旧实现：`../client/books`（Electron，保留不动）
- bundle identifier：`com.edtib.books`
- 产品名：紧固助手（个人版）
- 版本：1.0.0

## 技术栈

| 层 | 选型 |
| --- | --- |
| 桌面运行时 | Tauri v2（Rust） |
| 前端框架 | Vue 3 + Element Plus + Pinia |
| 构建工具 | Vite 8 + TypeScript 7（`vue-tsc` 类型检查） |
| 本地存储 | SQLite（`rusqlite`）+ 字段级 AES-256-CBC 加密 |
| 包管理 | npm |

## 前置环境

| 组件 | 本机已验证版本 |
| --- | --- |
| Node.js | v24.19.0 |
| npm | 11.17.0 |
| Rust（rustc / cargo） | 1.98.0 |
| Tauri CLI | 2.12.1（项目内 devDependency，通过 `npm run tauri` / `npx tauri` 调用，未做全局安装） |

> 若 shell 中提示 `node` / `cargo` 命令不存在，通常是未加载版本管理器的 PATH。Node 位于 `~/.nvm/versions/node/<ver>/bin`，Cargo 位于 `~/.cargo/bin`。

## 常用命令

```bash
npm install            # 安装前端依赖

npm run dev            # 仅启动前端 dev server（http://localhost:1420）
npm run tauri dev      # 启动完整桌面应用（前端 + Rust 热重载）

npm run typecheck      # 前端类型检查（vue-tsc --noEmit）
npm run build          # 前端类型检查 + 生产构建（输出 dist/）
npm run tauri build    # 打包桌面安装包（产物见 src-tauri/target/release/bundle/）

cd src-tauri && cargo test   # Rust 单测（加密 / 行转换 / 标识符防注入 / cron）

npx tauri icon <1024x1024.png>   # 由源图重新生成全套应用图标
```

> 与 console 相同，`vite build` 依赖商业模板授权码（`VITE_APP_GITHUB_USER_NAME` / `VITE_APP_SECRET_KEY`，
> 由 `.env` 与 `.env.local` 提供）：缺失时插件链仍完整注册但会打印授权无效提示，需以 `dist/` 是否产出为准。

## 运行时密钥

Rust 侧源码不含任何密钥（AGENTS §6.3）：

| 变量 | 作用 | 缺省行为 |
| --- | --- | --- |
| `EDTIB_DB_KEY_HEX` | 64 位十六进制 SQLCipher 主密钥（调试 / 自动化场景显式指定） | 未设置时按 Keychain → `<AppData>/.sqlcipher-key` 顺序取用或生成 |
| `EDTIB_FIELD_CRYPT_SECRET` / `EDTIB_FIELD_CRYPT_IV` | 本地库字段级 AES-256-CBC 密钥（`sha256` 派生 key/iv） | **未配置 `EDTIB_FIELD_CRYPT_SECRET` 时字段加解密关闭，字段以明文透传**；显式配置 IV 即进入与老 Electron 数据互读写的固定 IV 兼容模式，否则写出随机 IV 前置（`hex(iv‖cipher)`） |
| `EDTIB_DATA_DIR` / `EDTIB_LEGACY_DATA_DIR` | 覆盖应用数据目录（测试与自定义部署） | 平台默认 AppData 目录 |
| `EDTIB_UPDATER_ENDPOINT` | 更新清单地址（逗号/换行分隔可多个） | 回落到 `tauri.conf.json` 的 `plugins.updater.endpoints`；两者都缺省时 `check_update` 明确报错而非静默 |
| `EDTIB_VERSION_ALIAS` / `EDTIB_VERSION_TIMER` | 构建期注入，拼出老版 `getAppVersion` 文案 | 未注入时回落到 `CARGO_PKG_VERSION` |

> Keychain 中若已存在**非法**密钥记录，程序不会覆盖重写（避免既有加密库永久失读），而是回退同目录密钥文件，仍不可用则报错退出等待人工处理。
> 本地链路凭证（`appId` / `appSecret` / `appIv`）由前端 `VITE_APP_*` 在构建期注入，签名与加解密在 `src/utils/tauriHttp.ts` 完成后交给 Rust 出网。
> 完整变量说明见 `.env.example`。

## 目录结构

```
books/
├── index.html              # Vite 入口 HTML
├── package.json            # 前端依赖与脚本
├── vite.config.ts          # Vite 配置（固定端口 1420，忽略 src-tauri）
├── tsconfig.json           # 前端 TS 配置
├── tsconfig.node.json
├── .env.example            # 前端构建期 + Rust 运行期变量说明
├── library/                # 模板自带的构建插件链与通用组件（vab）
├── src/                    # 前端源码（Vue 3 + TS）
│   ├── main.ts             # 入口：setupBridge() 先于 createApp 注入 window.electronAPI
│   ├── bridge/             # channels / types / index —— Tauri invoke 桥接层
│   ├── api/                # 业务接口（经 request.ts 或 requestBridge 的 IPC 优先路径）
│   ├── utils/              # request.ts、requestBridge.ts、tauriHttp.ts、storage.ts、host 判定
│   ├── views/              # 60 个业务页面
│   └── router/ store/ i18n/ config/ plugins/ services/ types/
└── src-tauri/              # Rust 侧（Tauri）
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json             # 主配置
    ├── tauri.macos.conf.json       # macOS 平台覆盖
    ├── tauri.windows.conf.json     # Windows 平台覆盖
    ├── capabilities/default.json   # 权限能力集
    ├── icons/                      # 应用图标
    └── src/
        ├── main.rs / lib.rs        # 进程入口与插件、状态、命令注册
        ├── config（state.rs）/ error.rs / paths.rs
        ├── crypt.rs                # md5 / sha256 / aes-256-cbc（随机 IV 前置写出）
        ├── keychain.rs             # SQLCipher 主密钥：Keychain 优先，非法记录不覆盖
        ├── http.rs                 # 出网请求（App-Secret / X-Sign / 加解密口径）
        ├── update.rs               # 在线更新：下载 → 暂存 → 安装
        ├── commands/               # app / db / fs / http / system
        └── db/                     # connection / schema / query / execute / row / models
```

## 打包目标

| 平台 | 目标 | 说明 |
| --- | --- | --- |
| macOS | `app`、`dmg` | `.app` 应用包 + `.dmg` 安装镜像 |
| Windows | `nsis` | `.exe` 安装程序 |

> **注意**：Tauri v2 支持的内置打包目标为 `deb` / `rpm` / `appimage` / `msi` / `nsis` / `app` / `dmg`，**不包含 `zip`**。旧版 Electron 配置中的 macOS `zip` 目标在 Tauri 下没有等价项；如需 zip 分发，需在打包后自行对 `.app` 压缩（例如 `ditto -c -k --keepParent`），或另行引入 CI 后置步骤。
>
> 平台差异通过平台专属配置文件承载：macOS 构建自动合并 `tauri.macos.conf.json`，Windows 构建自动合并 `tauri.windows.conf.json`。

## 迁移状态

- [x] Tauri v2 + Vue3 + Vite + TS 骨架搭建
- [x] 渲染层业务代码迁入（`src/` 326 文件，含 60 个 `views/*.vue`）
- [x] Rust 侧命令层（`commands/{app,db,fs,http,system}.rs`）与桥接层（`src/bridge/`）
- [x] 本地存储：SQLite（`rusqlite`）+ SQLCipher 主密钥（Keychain / 密钥文件）+ 字段级加密
- [x] 与 gateway 的同步协议：签名头（`App-Secret` / `X-Sign`）+ `iv:cipher` 请求/响应加解密
- [x] 在线更新：检查 → 下载（进度事件）→ 确认安装 → 重启
- [ ] `tauri.conf.json` 待补：`plugins.updater`（`pubkey` + `endpoints`）与 `app.security.csp`——需项目负责人提供签名公钥与更新地址后再落地
- [ ] `.env.local` 授权码与 `VITE_APP_*` 凭证由发布机注入，仓库仅提供 `.env.example`
