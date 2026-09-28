# 紧固助手（个人版）— 桌面客户端

edtib books 桌面端由 **Electron 迁移至 Tauri v2** 的新项目。当前仓库仅包含**项目骨架**，尚未迁移任何业务代码。

- 旧实现：`../client/books`（Electron，保留不动）
- bundle identifier：`com.edtib.books`
- 产品名：紧固助手（个人版）
- 版本：1.0.0

## 技术栈

| 层 | 选型 |
| --- | --- |
| 桌面运行时 | Tauri v2（Rust） |
| 前端框架 | Vue 3 |
| 构建工具 | Vite 7 + TypeScript 5.9 |
| 包管理 | npm |

## 前置环境

| 组件 | 本机已具备版本 |
| --- | --- |
| Node.js | v24.21.0 |
| npm | 11.19.0 |
| Rust（rustc / cargo） | 1.97.1 |
| Tauri CLI | 2.12.0（项目内 devDependency，通过 `npm run tauri` / `npx tauri` 调用，未做全局安装） |

> 若 shell 中提示 `node` / `cargo` 命令不存在，通常是未加载版本管理器的 PATH。Node 位于 `~/.nvm/versions/node/<ver>/bin`，Cargo 位于 `~/.cargo/bin`。

## 常用命令

```bash
npm install            # 安装前端依赖

npm run dev            # 仅启动前端 dev server（http://localhost:1420）
npm run tauri dev      # 启动完整桌面应用（前端 + Rust 热重载）

npm run build          # 前端类型检查 + 生产构建（输出 dist/）
npm run tauri build    # 打包桌面安装包（产物见 src-tauri/target/release/bundle/）

npx tauri icon <1024x1024.png>   # 由源图重新生成全套应用图标
```

## 目录结构

```
books/
├── index.html              # Vite 入口 HTML
├── package.json            # 前端依赖与脚本
├── vite.config.ts          # Vite 配置（固定端口 1420，忽略 src-tauri）
├── tsconfig.json           # 前端 TS 配置
├── tsconfig.node.json
├── src/                    # 前端源码（Vue 3 + TS）
│   ├── main.ts
│   ├── App.vue
│   └── vite-env.d.ts
└── src-tauri/              # Rust 侧（Tauri）
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json             # 主配置
    ├── tauri.macos.conf.json       # macOS 平台覆盖
    ├── tauri.windows.conf.json     # Windows 平台覆盖
    ├── capabilities/default.json   # 权限能力集
    ├── icons/                      # 应用图标（已生成）
    └── src/
        ├── main.rs
        └── lib.rs                  # 含 greet 示例命令，用于验证 IPC 通道
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
- [ ] 业务模块迁移（渲染层、Rust 侧命令、本地存储、与 gateway 的同步协议）—— 尚未开始
*（内容由AI生成，仅供参考）*
