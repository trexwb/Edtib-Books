/**
 * @description 序列号/优惠券 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:35
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 已经用优惠券
 * IPC: serials 表（本地记录）
 * HTTP: /front/customers/serialsUse
 */
export async function serialsUse(data: {
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
      table: 'serials',
      filters: data.filter || { status: 'used' },
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/customers/serialsUse',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 兑换优惠券
 * IPC: 不适用（需要在线验证）
 * HTTP: /front/customers/serialsExchange
 */
export async function serialsExchange(data: { code?: string }) {
  // 兑换操作必须在线验证，不使用 IPC fallback
  return request({
    url: '/front/customers/serialsExchange',
    method: 'post',
    data,
  })
}

export { requestBridge }