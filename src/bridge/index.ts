/**
 * @description 桥接模块（Electron IPC / preload 调用点统一收敛层）
 *
 * 迁移背景：
 *   旧版本由 Electron preload 通过 contextBridge 暴露 window.electronAPI（db / fs / system /
 *   更新与缓存等能力）。迁移到 Tauri 后主进程能力由 Rust 侧 command 提供，
 *   本模块把同一套接口（src/bridge/types.ts 中的 ElectronAPI）统一转发到 Tauri invoke，
 *   使上层业务代码无需感知运行时差异。
 *
 * 使用方式：
 *   1) 推荐：`import { bridge } from '/@/bridge'` 后直接调用 bridge.db.findAll(...)；
 *   2) 兼容：应用启动（src/main.ts）会调用 setupBridge()，在 Tauri 环境下把 bridge 挂到
 *      window.electronAPI，历史代码中的 window.electronAPI.xxx 依然可用。
 *
 * 占位实现说明（当前阶段）：
 *   Rust 侧 command 尚未实现，因此：
 *   - db / fs / cacheFile 等"数据能力"：invoke 失败即抛错，由调用方（requestBridge）按既有逻辑
 *     降级为 HTTP 请求，行为与在线模式一致；
 *   - getAppVersion / checkUpdate / restartApp / system.* 等"环境能力"：invoke 失败时返回安全
 *     兜底值，保证 UI 不因未实现而报错；
 *   - 更新事件：Tauri 环境下已接好事件监听通道（BRIDGE_EVENTS），Rust 侧 emit 后即生效。
 *   所有调用点均已保留，待 Rust 侧按 src/bridge/channels.ts 的命令名实现后即可自动生效。
 * @Author: trexwb
 */

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { BRIDGE_CHANNELS, BRIDGE_EVENTS } from './channels'
import type { ElectronAPI, SystemInfo, UpdateInfo } from './types'

export * from './types'
export { BRIDGE_CHANNELS, BRIDGE_EVENTS, ELECTRON_IPC_CHANNEL_MAP } from './channels'

/** 应用版本兜底值（Rust 侧 get_app_version 实现前使用；与 package.json / tauri.conf.json 保持一致） */
const APP_VERSION_FALLBACK = '1.0.0'

/** 判断是否运行在 Tauri 环境（Tauri v2 注入 __TAURI_INTERNALS__） */
export const isTauri = (): boolean => {
  if (typeof window === 'undefined') return false
  const w = window as any
  return Boolean(w.__TAURI_INTERNALS__ || w.__TAURI__)
}

/** 桥接能力是否可用（等价于旧实现的 isElectron()，供上层做能力判断） */
export const isBridgeAvailable = (): boolean => isTauri()

/** 已告警过的命令，避免未实现命令重复刷日志 */
const warnedCommands = new Set<string>()

const warnNotImplemented = (command: string, error: unknown) => {
  if (warnedCommands.has(command)) return
  warnedCommands.add(command)
  console.warn(
    `[bridge] Tauri 命令 "${command}" 暂未实现或调用失败，已按占位策略降级。` +
      'Rust 侧实现后自动生效，通道对照见 src/bridge/channels.ts',
    error
  )
}

/**
 * 统一 invoke 封装：非 Tauri 环境直接 reject，Tauri 环境转发到 Rust command
 */
const call = <T>(command: string, payload?: Record<string, any>): Promise<T> => {
  if (!isTauri()) {
    return Promise.reject(new Error(`[bridge] 当前非 Tauri 环境，命令 ${command} 不可用`))
  }
  return payload === undefined ? invoke<T>(command) : invoke<T>(command, payload)
}

/** 更新事件监听注册表（登记 entry 以便注销，避免重复注册导致同一回调多次触发） */
interface UpdateListenerEntry {
  event: string
  callback: (event: unknown, payload: any) => void
  unlisten?: () => void
}
const updateListeners: UpdateListenerEntry[] = []

const registerUpdateListener = (event: string, callback: (event: unknown, payload: any) => void): (() => void) => {
  if (typeof callback !== 'function') return () => {}
  const entry: UpdateListenerEntry = { callback, event }
  updateListeners.push(entry)
  if (isTauri()) {
    // Tauri 环境下同时挂到事件总线：Rust 侧按 BRIDGE_EVENTS 定义 emit 即生效
    void listen(event, (tauriEvent) => callback(tauriEvent, (tauriEvent as any)?.payload))
      .then((unlisten) => {
        entry.unlisten = unlisten
      })
      .catch((error) => {
        warnNotImplemented(event, error)
      })
  }
  return () => {
    const index = updateListeners.indexOf(entry)
    if (index !== -1) updateListeners.splice(index, 1)
    entry.unlisten?.()
  }
}

/**
 * 供 Rust 侧事件接入完成后手动派发（占位阶段保留，便于联调）
 */
export const dispatchBridgeEvent = (event: string, payload?: any) => {
  updateListeners.filter((item) => item.event === event).forEach((item) => item.callback({ event }, payload))
}

/** 浏览器环境系统信息兜底（Tauri system_get_info 实现前的安全返回值） */
const fallbackSystemInfo = (): SystemInfo => {
  const ua = typeof navigator !== 'undefined' ? navigator.userAgent || '' : ''
  const platform = /mac/i.test(ua) ? 'darwin' : /win/i.test(ua) ? 'win32' : /linux/i.test(ua) ? 'linux' : 'unknown'
  return {
    platform,
    arch: 'unknown',
    hostname: '',
    cpus: typeof navigator !== 'undefined' ? navigator.hardwareConcurrency || 0 : 0,
    totalMemory: '',
    freeMemory: '',
    versions: {
      node: '',
      chrome: (ua.match(/Chrome\/([\d.]+)/) || [])[1] || '',
      electron: '',
    },
    webview: (ua.match(/(?:AppleWebKit|WebKit)\/([\d.]+)/) || [])[1] || '',
  }
}

/**
 * 桥接实现（与 Electron preload 暴露的接口同构）
 */
export const bridge: ElectronAPI = {
  // === 版本与更新 ===
  getAppVersion: async (): Promise<string> => {
    try {
      return (await call<string>(BRIDGE_CHANNELS.app.getAppVersion)) || APP_VERSION_FALLBACK
    } catch (error) {
      warnNotImplemented(BRIDGE_CHANNELS.app.getAppVersion, error)
      return APP_VERSION_FALLBACK
    }
  },
  checkUpdate: async (): Promise<void> => {
    try {
      await call(BRIDGE_CHANNELS.app.checkUpdate)
    } catch (error) {
      // 占位：Rust 侧 updater 未接入前静默降级，避免点击检查更新即报错
      warnNotImplemented(BRIDGE_CHANNELS.app.checkUpdate, error)
    }
  },
  confirmUpdate: async (): Promise<void> => {
    // [迁移补全] Rust 侧 confirm_update 已注册（lib.rs），负责安装 check_update 已下载的包；
    // 此前桥接层未暴露该方法，更新包下载完成后永远无法安装。
    await call(BRIDGE_CHANNELS.app.confirmUpdate)
  },
  restartApp: (): void => {
    // 接口签名为同步（沿用 Electron 版），内部异步触发
    void call(BRIDGE_CHANNELS.app.restartApp).catch((error) => {
      warnNotImplemented(BRIDGE_CHANNELS.app.restartApp, error)
    })
  },
  onUpdateAvailable: (callback) => registerUpdateListener(BRIDGE_EVENTS.updateAvailable, callback),
  onUpdateNotAvailable: (callback) => registerUpdateListener(BRIDGE_EVENTS.updateNotAvailable, callback),
  onDownloadProgress: (callback) => registerUpdateListener(BRIDGE_EVENTS.downloadProgress, callback),
  onUpdateDownloaded: (callback) => registerUpdateListener(BRIDGE_EVENTS.updateDownloaded, callback),
  cacheFile: (filePath: string): Promise<string> => call<string>(BRIDGE_CHANNELS.app.cacheFile, { url: filePath }),

  // === 数据库（失败即抛错，由 requestBridge 降级为 HTTP） ===
  db: {
    findAll: (table: string, filters?: Record<string, any>) =>
      call<any[] | null>(BRIDGE_CHANNELS.db.findAll, { table, filters }),
    getList: (table: string, filters?: Record<string, any>, order?: any[], limit?: number, offset?: number) =>
      call<{ total: number; list: any[] }>(BRIDGE_CHANNELS.db.getList, { table, filters, order, limit, offset }),
    findOne: (table: string, id: number | string) => call<any | null>(BRIDGE_CHANNELS.db.findOne, { table, id }),
    create: (table: string, data: Record<string, any>) => call<any | null>(BRIDGE_CHANNELS.db.create, { table, data }),
    update: (table: string, id: number | string, data: Record<string, any>) =>
      call<any | null>(BRIDGE_CHANNELS.db.update, { table, id, data }),
    delete: (table: string, id: number | string) => call<any | null>(BRIDGE_CHANNELS.db.delete, { table, id }),
    bulkCreate: (table: string, data: Record<string, any>[]) =>
      call<any | null>(BRIDGE_CHANNELS.db.bulkCreate, { table, data }),
  },

  // === 文件系统（失败即抛错，由 requestBridge 降级为 HTTP） ===
  fs: {
    readFile: (filePath: string) => call<string | null>(BRIDGE_CHANNELS.fs.readFile, { filePath }),
    writeFile: (filePath: string, data: string) => call<string | null>(BRIDGE_CHANNELS.fs.writeFile, { filePath, data }),
    deleteFile: (filePath: string) => call<boolean>(BRIDGE_CHANNELS.fs.deleteFile, { filePath }),
    listFiles: (dir: string) => call<string[]>(BRIDGE_CHANNELS.fs.listFiles, { dir }),
    getUserDataPath: () => call<string>(BRIDGE_CHANNELS.fs.getUserDataPath),
    getDocumentsPath: () => call<string>(BRIDGE_CHANNELS.fs.getDocumentsPath),
    exists: (filePath: string) => call<boolean>(BRIDGE_CHANNELS.fs.exists, { filePath }),
  },

  // === 系统信息 ===
  system: {
    getPlatform: (): string => fallbackSystemInfo().platform,
    getArch: (): string => fallbackSystemInfo().arch,
    getVersions: () => {
      const info = fallbackSystemInfo()
      return { node: info.versions.node, chrome: info.versions.chrome, electron: info.versions.electron }
    },
    getInfo: async (): Promise<SystemInfo> => {
      try {
        return await call<SystemInfo>(BRIDGE_CHANNELS.system.getInfo)
      } catch (error) {
        warnNotImplemented(BRIDGE_CHANNELS.system.getInfo, error)
        return fallbackSystemInfo()
      }
    },
  },
}

let installed = false

/**
 * 在 Tauri 环境下把桥接实现挂载为 window.electronAPI（兼容历史调用点，幂等）
 */
export const setupBridge = (): ElectronAPI => {
  if (typeof window !== 'undefined' && isTauri() && !installed) {
    ;(window as any).electronAPI = bridge
    installed = true
    console.info('[bridge] Electron IPC -> Tauri invoke 桥接已安装（window.electronAPI）')
  }
  return bridge
}

/** 别名，语义同上 */
export const installBridge = setupBridge
