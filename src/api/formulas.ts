/**
 * @description 公式 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:30
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 公式列表（所有）
 * IPC: formulas 表
 * HTTP: /front/standards/formulasAll
 */
export async function formulasAll() {
  const result = await requestBridge.getAll({
    ipc: {
      table: 'formulas',
      filters: { status: 1 },
    },
    http: {
      url: '/front/standards/formulasAll',
    },
  })
  return { data: result }
}

export { requestBridge }