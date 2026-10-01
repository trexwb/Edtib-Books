/**
 * @description 桥接层类型定义（Electron -> Tauri 迁移唯一收敛点）
 *
 * 说明：
 *  - 原 Electron 版本中这些接口以全局声明形式维护在 src/types/electron.d.ts；
 *    迁移后统一收敛到本文件，electron.d.ts 只保留 Window 全局挂载声明并从此处 re-export。
 *  - 接口签名与旧 Electron preload（electron/src/preload.ts）保持完全一致，
 *    以保证上层业务代码零改动即可切换到 Tauri invoke 实现。
 * @Author: trexwb
 */

/** 更新信息（字段与 Rust update.rs::UpdateInfo::to_json 对齐） */
export interface UpdateInfo {
  version: string
  /** [迁移新增] 当前运行版本 */
  currentVersion?: string
  /** [迁移新增] 发布日期，Rust 侧字段名为 date */
  date?: string
  releaseNotes?: string
  /** [迁移新增] 更新目标（platform_arch，如 darwin_aarch64） */
  target?: string
}

/** 下载进度载荷（与 Rust EVENT_DOWNLOAD_PROGRESS 的 emit 结构对齐，Rust 发对象而非裸数字） */
export interface DownloadProgress {
  percent: number
  transferred: number
  total: number | null
}

/** 系统信息 */
export interface SystemInfo {
  platform: string
  arch: string
  hostname: string
  cpus: number
  totalMemory: string
  freeMemory: string
  versions: {
    node: string
    chrome: string
    electron: string
  }
  /** [迁移新增] Tauri 下由系统 WebView 承载，补充可选字段，不破坏既有字段契约 */
  webview?: string
}

/**
 * 桥接接口（与 Electron preload 暴露的 window.electronAPI 完全同构）
 */
export interface ElectronAPI {
  // === 版本与更新 API ===
  getAppVersion: () => Promise<string>
  checkUpdate: () => Promise<void>
  /** [迁移补全] 安装已下载的更新包（对应 Rust confirm_update）；老 Electron 由 quitAndInstall 承担 */
  confirmUpdate: () => Promise<void>
  restartApp: () => void
  // 注意：以下事件回调均源自 ipcRenderer.on，首个参数为 IpcRendererEvent
  // 返回值 [迁移调整] 为注销函数，组件卸载时调用，避免重复注册导致回调多次触发
  onUpdateAvailable: (callback: (event: unknown, info: UpdateInfo) => void) => () => void
  onUpdateNotAvailable: (callback: (event: unknown, info: UpdateInfo) => void) => () => void
  onDownloadProgress: (callback: (event: unknown, payload: DownloadProgress) => void) => () => void
  onUpdateDownloaded: (callback: (event: unknown, info: UpdateInfo) => void) => () => void
  cacheFile: (filePath: string) => Promise<string>

  // === 数据库操作 API ===
  db: {
    findAll: (table: string, filters?: Record<string, any>) => Promise<any[] | null>
    getList: (
      table: string,
      filters?: Record<string, any>,
      order?: any[],
      limit?: number,
      offset?: number
    ) => Promise<{ total: number; list: any[] }>
    findOne: (table: string, id: number | string) => Promise<any | null>
    create: (table: string, data: Record<string, any>) => Promise<any | null>
    update: (table: string, id: number | string, data: Record<string, any>) => Promise<any | null>
    delete: (table: string, id: number | string) => Promise<any | null>
    bulkCreate: (table: string, data: Record<string, any>[]) => Promise<any | null>
  }

  // === 文件系统 API ===
  fs: {
    readFile: (filePath: string) => Promise<string | null>
    writeFile: (filePath: string, data: string) => Promise<string | null>
    deleteFile: (filePath: string) => Promise<boolean>
    listFiles: (dir: string) => Promise<string[]>
    getUserDataPath: () => Promise<string>
    getDocumentsPath: () => Promise<string>
    exists: (filePath: string) => Promise<boolean>
  }

  // === 系统信息 API ===
  system: {
    getPlatform: () => string
    getArch: () => string
    getVersions: () => { node: string; chrome: string; electron: string }
    getInfo: () => Promise<SystemInfo>
  }
}

/** 更新事件回调签名（供 bridge 内部注册使用） */
export type UpdateEventCallback = (event: unknown, payload: any) => void
