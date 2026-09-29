/**
 * @description 离线优先请求桥接器
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:00
 */

import request from '/@/utils/request'
// [迁移调整] 统一走 src/bridge 桥接层：Tauri 环境下由 invoke 承载，Electron 兼容层同源
import { bridge, isBridgeAvailable } from '/@/bridge'

// C-B4: IPC 方法白名单，仅允许显式声明的受控方法，杜绝 __proto__/constructor 等原型链或危险方法注入
const DB_METHODS: ReadonlySet<string> = new Set(['getList', 'findOne', 'findAll', 'create', 'update', 'delete'])
const FS_METHODS: ReadonlySet<string> = new Set(['readFile', 'writeFile', 'listFiles'])
const SYSTEM_METHODS: ReadonlySet<string> = new Set(['getInfo'])

// 检测本地 IPC 能力是否可用（迁移语义对齐：旧版判断 window.electronAPI，
// 现统一由 src/bridge 判定 —— Tauri 环境为 invoke 桥接，Electron 环境为兼容层）
const isElectron = (): boolean => {
  return isBridgeAvailable()
}

// IPC 调用超时时间（毫秒）
const IPC_TIMEOUT = 5000

// 序列化参数，防止 IPC 传递过程中对象引用被意外修改
const cloneForIpc = (value: any): any => {
  if (value === undefined || value === null) return value
  try {
    return JSON.parse(JSON.stringify(value))
  } catch {
    return value
  }
}

// IPC 数据库调用（直接调用 electronAPI.db[method]，而非通过 electronAPI[channel] 函数调用）
const ipcDbCall = async (method: string, table: string, ...args: any[]): Promise<any> => {
  if (!DB_METHODS.has(method)) {
    throw new Error(`Disallowed IPC db method: ${method}`)
  }
  if (!isElectron()) {
    throw new Error('Not in Electron environment')
  }

  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`IPC db.${method} timeout after ${IPC_TIMEOUT}ms`))
    }, IPC_TIMEOUT)

    // [迁移调整] 统一走 src/bridge 桥接层（Tauri invoke）
    const dbApi = bridge.db as unknown as Record<string, (...invokeArgs: any[]) => Promise<any>>
    dbApi[method](table, ...args.map(cloneForIpc))
      .then((result: any) => {
        clearTimeout(timeoutId)
        resolve(result)
      })
      .catch((error: any) => {
        clearTimeout(timeoutId)
        reject(error)
      })
  })
}

// IPC 文件系统调用（直接调用 electronAPI.fs[method]）
const ipcFsCall = async (method: string, ...args: any[]): Promise<any> => {
  if (!FS_METHODS.has(method)) {
    throw new Error(`Disallowed IPC fs method: ${method}`)
  }
  if (!isElectron()) {
    throw new Error('Not in Electron environment')
  }

  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`IPC fs.${method} timeout after ${IPC_TIMEOUT}ms`))
    }, IPC_TIMEOUT)

    // [迁移调整] 统一走 src/bridge 桥接层（Tauri invoke）
    const fsApi = bridge.fs as unknown as Record<string, (...invokeArgs: any[]) => Promise<any>>
    fsApi[method](...args.map(cloneForIpc))
      .then((result: any) => {
        clearTimeout(timeoutId)
        resolve(result)
      })
      .catch((error: any) => {
        clearTimeout(timeoutId)
        reject(error)
      })
  })
}

// IPC 系统调用（直接调用 electronAPI.system[method]）
const ipcSystemCall = async (method: string, ...args: any[]): Promise<any> => {
  if (!SYSTEM_METHODS.has(method)) {
    throw new Error(`Disallowed IPC system method: ${method}`)
  }
  if (!isElectron()) {
    throw new Error('Not in Electron environment')
  }

  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`IPC system.${method} timeout after ${IPC_TIMEOUT}ms`))
    }, IPC_TIMEOUT)

    // [迁移调整] 统一走 src/bridge 桥接层（Tauri invoke）
    const systemApi = bridge.system as unknown as Record<string, (...invokeArgs: any[]) => Promise<any>>
    systemApi[method](...args.map(cloneForIpc))
      .then((result: any) => {
        clearTimeout(timeoutId)
        resolve(result)
      })
      .catch((error: any) => {
        clearTimeout(timeoutId)
        reject(error)
      })
  })
}

/**
 * 离线优先请求桥接器
 * 优先 IPC，失败时 fallback 到 HTTP
 */
const requestBridge = {
  /**
   * 获取列表数据（分页）
   * IPC: db.getList(table, filters, order, limit, offset)
   * HTTP: request({ url, method: 'post', data })
   */
  getList: async (options: {
    ipc?: {
      table: string
      filters?: any
      order?: any
      limit?: number
      offset?: number
    }
    http?: {
      url: string
      data?: any
    }
  }): Promise<{ total: number; list: any[] }> => {
    // 优先 IPC
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall(
          'getList',
          options.ipc.table,
          options.ipc.filters,
          options.ipc.order,
          options.ipc.limit,
          options.ipc.offset
        )
        const data = result.data || result
        if (data && typeof data.total === 'number') {
          return { total: data.total || 0, list: data.list || [] }
        }
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data || { total: 0, list: [] }
    }

    return { total: 0, list: [] }
  },

  /**
   * 获取单条数据
   * IPC: db.findOne(table, id)
   * HTTP: request({ url, method: 'post', data: { id } })
   */
  getOne: async (options: {
    ipc?: {
      table: string
      id?: number | string
    }
    http?: {
      url: string
      data: { id?: number | number[] }
    }
  }): Promise<any> => {
    // 优先 IPC
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall('findOne', options.ipc.table, options.ipc.id)
        const data = result.data || result
        if (data !== null && data !== undefined && (typeof data !== 'object' || Object.keys(data).length > 0)) {
          return data
        }
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data
    }

    return null
  },

  /**
   * 获取所有数据
   * IPC: db.findAll(table, filters)
   * HTTP: request({ url, method: 'post', data })
   */
  // 返回类型说明：IPC 路径返回数组，HTTP 路径可能返回 { list } 包裹结构，由调用方自行兼容
  getAll: async (options: {
    ipc?: {
      table: string
      filters?: any
    }
    http?: {
      url: string
      data?: any
    }
  }): Promise<any> => {
    // 优先 IPC
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall('findAll', options.ipc.table, options.ipc.filters)
        const data = result.data || result
        if (data && Array.isArray(data) && data.length > 0) {
          return data
        }
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data || []
    }

    return []
  },

  /**
   * 创建数据
   * IPC: db.create(table, data)
   * HTTP: request({ url, method: 'post', data })
   */
  create: async (options: {
    ipc?: {
      table: string
      data: any
    }
    http?: {
      url: string
      data: any
    }
  }): Promise<any> => {
    // 优先 IPC（离线写入）
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall('create', options.ipc.table, options.ipc.data)
        return result.data || result
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data
    }

    return null
  },

  /**
   * 更新数据
   * IPC: db.update(table, id, data)
   * HTTP: request({ url, method: 'post', data })
   */
  update: async (options: {
    ipc?: {
      table: string
      id: number | string
      data: any
    }
    http?: {
      url: string
      data: any
    }
  }): Promise<any> => {
    // 优先 IPC（离线更新）
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall('update', options.ipc.table, options.ipc.id, options.ipc.data)
        return result.data || result
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data
    }

    return null
  },

  /**
   * 删除数据
   * IPC: db.delete(table, id)
   * HTTP: request({ url, method: 'post', data })
   */
  delete: async (options: {
    ipc?: {
      table: string
      id: number | string
    }
    http?: {
      url: string
      data: { id: number | number[] }
    }
  }): Promise<any> => {
    // 优先 IPC（离线删除）
    if (options.ipc && isElectron()) {
      try {
        const result = await ipcDbCall('delete', options.ipc.table, options.ipc.id)
        return result.data || result
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'post',
        data: options.http.data,
      })
      return data
    }

    return null
  },

  /**
   * 文件读取
   * IPC: fs.readFile(filePath)
   * HTTP: 请求远程文件
   */
  readFile: async (options: {
    ipc?: {
      filePath: string
    }
    http?: {
      url: string
    }
  }): Promise<string | null> => {
    // 优先 IPC（本地文件）
    if (options.ipc && isElectron()) {
      try {
        const response = await ipcFsCall('readFile', options.ipc.filePath)
        if (response && response.code === 200) {
          return response.data
        }
      } catch (error) {
        console.warn('[IPC fallback to HTTP]', error)
      }
    }

    // Fallback HTTP（远程文件）
    if (options.http) {
      const { data } = await request({
        url: options.http.url,
        method: 'get',
      })
      return data
    }

    return null
  },

  /**
   * 文件写入（仅 IPC）
   * HTTP 不支持文件写入，仅本地
   */
  writeFile: async (options: {
    ipc: {
      filePath: string
      data: string
    }
  }): Promise<string | null> => {
    if (!isElectron()) {
      console.warn('[writeFile] Not in Electron environment')
      return null
    }

    try {
      return await ipcFsCall('writeFile', options.ipc.filePath, options.ipc.data)
    } catch (error) {
      console.error('[writeFile failed]', error)
      return null
    }
  },

  /**
   * 列出目录文件（仅 IPC）
   * IPC: fs.listFiles(dir)
   */
  listFiles: async (options: {
    ipc: {
      dir: string
    }
  }): Promise<string[]> => {
    if (!isElectron()) {
      console.warn('[listFiles] Not in Electron environment')
      return []
    }

    try {
      const result = await ipcFsCall('listFiles', options.ipc.dir)
      return (result && (result.data || result)) || []
    } catch (error) {
      console.error('[listFiles failed]', error)
      return []
    }
  },

  /**
   * 获取系统信息
   * IPC: system.getInfo()
   */
  getSystemInfo: async (): Promise<any> => {
    if (!isElectron()) {
      return {
        platform: 'web',
        arch: 'unknown',
        hostname: 'unknown',
      }
    }

    try {
      return await ipcSystemCall('getInfo')
    } catch (error) {
      console.error('[getSystemInfo failed]', error)
      return null
    }
  },

  // 工具方法
  isElectron,
  ipcDbCall,
  ipcFsCall,
  ipcSystemCall,
}

export default requestBridge
