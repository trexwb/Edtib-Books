/**
 * @description 公共配置 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:45
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 环境配置常量
 * IPC: enums 表或 fs.readFile（本地缓存）
 * HTTP: /front/common/configs
 */
export async function configEnum() {
  // 优先读取本地缓存
  const cached = await requestBridge.readFile({
    ipc: { filePath: 'common/config-cache.json' },
  })
  
  if (cached) {
    return { data: JSON.parse(cached) }
  }
  
  // 尝试从 enums 表获取
  const enums = await requestBridge.getAll({
    ipc: {
      table: 'enums',
      filters: { status: 1 },
    },
  })
  
  if (enums && enums.length > 0) {
    return { data: { enums } }
  }
  
  // Fallback HTTP
  const { data } = await request({
    url: '/front/common/configs',
    method: 'post',
  })
  
  // 缓存结果
  if (data && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'common/config-cache.json', data: JSON.stringify(data) },
    })
  }
  
  return { data }
}

/**
 * 路由配置
 * IPC: fs.readFile（本地缓存）
 * HTTP: /front/common/routes
 */
export async function getRoutesPath() {
  // 优先读取本地缓存
  const cached = await requestBridge.readFile({
    ipc: { filePath: 'common/routes-cache.json' },
  })
  
  if (cached) {
    return { data: JSON.parse(cached) }
  }
  
  // Fallback HTTP
  const { data } = await request({
    url: '/front/common/routes',
    method: 'post',
  })
  
  // 缓存结果
  if (data && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'common/routes-cache.json', data: JSON.stringify(data) },
    })
  }
  
  return { data }
}

export { requestBridge }