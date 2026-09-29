/**
 * @description Electron API 全局类型声明
 *
 * [迁移调整] 接口定义已统一收敛至 src/bridge/types.ts（便于 Rust 侧对齐与替换），
 * 本文件仅保留 window.electronAPI 的全局挂载声明与向后兼容的类型再导出。
 * @Author: trexwb
 */

import type { ElectronAPI } from '/@/bridge/types'

export type { ElectronAPI, SystemInfo, UpdateInfo, UpdateEventCallback } from '/@/bridge/types'

declare global {
  interface Window {
    /** Electron 兼容层：由 src/bridge/setupBridge() 在 Tauri 环境下挂载 */
    electronAPI: ElectronAPI
  }
}

export {}
