/**
 * @description 资料下载 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:25
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 浏览记录
 * IPC: docs_browsers 表（如有）
 * HTTP: /front/standards/docsBrowsers
 */
export async function docsBrowsers(data: {
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
    // IPC 配置（如果本地有浏览记录表）
    ipc: {
      table: 'docs_browsers',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/docsBrowsers',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 下载记录
 * IPC: docs_logs 表（如有）
 * HTTP: /front/standards/docsLogs
 */
export async function docsLogs(data: {
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
    // IPC 配置（如果本地有下载记录表）
    ipc: {
      table: 'docs_logs',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/docsLogs',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 标准列表（分页）
 * 
 * @param data 文档临时数据对象
 * @param data.filter.keywords 模糊搜索
 * @param data.filter.id 编号搜索
 * @param data.filter.standard_id 标准分类编号
 * @param data.filter.shape_id 形状分类编号
 * @param data.filter.category_id 文档分类编号
 * @param data.filter.code 文档编码（可选）
 * @param data.filter.status 状态：0禁用，1启用（可选）
 * @param data.sort 排序方式（可选）
 * @param data.page 页码（可选）
 * @param data.pageSize 每页数量（可选）
 * @returns 返回请求的Promise对象，包含列表数据
 */
export async function docsList(data: {
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
      table: 'docs',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/docsList',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 文档原件下载
 * IPC: fs.readFile（本地缓存）
 * HTTP: /front/standards/docsDownload
 */
export async function docsDownload(data: { id?: number | number[] }) {
  // 优先检查本地缓存
  const cacheKey = `docs/${Array.isArray(data.id) ? data.id.join('-') : data.id}`
  const cached = await requestBridge.readFile({
    ipc: { filePath: cacheKey },
  })
  
  if (cached) {
    return { data: JSON.parse(cached) }
  }
  
  // Fallback HTTP
  const { data: result } = await request({
    url: '/front/standards/docsDownload',
    method: 'post',
    data,
  })
  
  // 缓存结果（仅 IPC 环境）
  if (result && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: cacheKey, data: JSON.stringify(result) },
    })
  }
  
  return { data: result }
}

/**
 * 文档详细
 * IPC: docs 表
 * HTTP: /front/standards/docsDetail
 */
export async function docsDetail(data: { id?: number | number[] }) {
  const id = Array.isArray(data.id) ? data.id[0] : data.id
  const result = await requestBridge.getOne({
    ipc: {
      table: 'docs',
      id,
    },
    http: {
      url: '/front/standards/docsDetail',
      data,
    },
  })
  return { data: result }
}

// 临时保留 HTTP request（用于 docsDownload 的 HTTP fallback）
import request from '/@/utils/request'

// 导出 requestBridge
export { requestBridge }