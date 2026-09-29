/**
 * @description 附件 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:45
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 获取签名（用于文件上传）
 * IPC: secrets 表（本地缓存签名配置）
 * HTTP: /front/attachments/getSign
 */
export async function getSign(data: { open?: 1 }) {
  // 优先读取本地缓存
  const cached = await requestBridge.readFile({
    ipc: { filePath: 'attachments/sign-cache.json' },
  })
  
  if (cached) {
    return { data: JSON.parse(cached) }
  }
  
  // Fallback HTTP
  const { data: result } = await request({
    url: '/front/attachments/getSign',
    method: 'post',
    data,
  })
  
  // 缓存结果
  if (result && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'attachments/sign-cache.json', data: JSON.stringify(result) },
    })
  }
  
  return { data: result }
}

export { requestBridge }