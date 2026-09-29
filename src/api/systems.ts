/**
 * @description 系统 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:35
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 最后更新时间
 * IPC: fs.readFile（本地记录）
 * HTTP: /front/systems/updateLastTime
 */
export async function updateLastTime() {
  // 优先读取本地记录
  const cached = await requestBridge.readFile({
    ipc: { filePath: 'system/update-last-time.json' },
  })
  
  if (cached) {
    return { data: JSON.parse(cached) }
  }
  
  // Fallback HTTP
  const { data } = await request({
    url: '/front/systems/updateLastTime',
    method: 'post',
  })
  
  // 缓存结果
  if (data && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'system/update-last-time.json', data: JSON.stringify(data) },
    })
  }
  
  return { data }
}

/**
 * 清空缓存
 * IPC: fs.deleteFile（清空本地缓存目录）
 * HTTP: /front/systems/cachesClear
 */
export async function cachesClear() {
  // 优先清空本地缓存
  if (requestBridge.isElectron()) {
    try {
      const files = await requestBridge.listFiles({ ipc: { dir: '' } })
      for (const file of files) {
        if (file.endsWith('.json') || file.startsWith('docs/') || file.startsWith('system/')) {
          await requestBridge.writeFile({
            ipc: { filePath: file, data: '' },
          })
        }
      }
    } catch (error) {
      console.warn('[cachesClear] IPC failed:', error)
    }
  }
  
  // 同时清空服务器缓存
  return request({
    url: '/front/systems/cachesClear',
    method: 'post',
  })
}

/**
 * 获取系统信息
 * IPC: system.getInfo
 */
export async function getSystemInfo() {
  const info = await requestBridge.getSystemInfo()
  return { data: info }
}

export { requestBridge }