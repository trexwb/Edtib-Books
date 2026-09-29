/**
 * @description 下载记录 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:40
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 下载记录
 * IPC: downloads_logs 表（如有）
 * HTTP: /front/standards/downloadsLogs
 */
export async function downloadsLogs(data: {
  filter?: object
  sort?: string
  page?: number
  pageSize?: number
}) {
  // 解析排序
  const order = data.sort
    ? [{ column: data.sort.replace(/^[+-]/, ''), order: data.sort.startsWith('-') ? 'DESC' : 'ASC' }]
    : undefined
  
  const limit = data.pageSize || 20
  const offset = ((data.page || 1) - 1) * limit
  
  const result = await requestBridge.getList({
    ipc: {
      table: 'downloads_logs',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/downloadsLogs',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 浏览记录
 * IPC: docs_hits 表（如有）
 * HTTP: /front/standards/docsHits
 */
export async function docsHits(data: {
  filter?: object
  sort?: string
  page?: number
  pageSize?: number
}) {
  // 解析排序
  const order = data.sort
    ? [{ column: data.sort.replace(/^[+-]/, ''), order: data.sort.startsWith('-') ? 'DESC' : 'ASC' }]
    : undefined
  
  const limit = data.pageSize || 20
  const offset = ((data.page || 1) - 1) * limit
  
  const result = await requestBridge.getList({
    ipc: {
      table: 'docs_hits',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/docsHits',
      data,
    },
  })
  
  return { data: result }
}

export { requestBridge }