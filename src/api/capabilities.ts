/**
 * @description 性能标准 API（免费内容：注册用户全部可见，无购买 / 订阅语义）
 *
 * 服务端当前无性能标准数据源，capabilitiesList 返回免费空态：
 * { standard_type:'capabilities', total:0, list:[], access:'free', available:false, message:'免费内容，数据接入中' }
 * 数据接入后前端无需改动（同构渲染 list）。
 *
 * @Author: trexwb
 * @Date: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 性能标准列表
 * HTTP: /front/standards/capabilitiesList
 * @returns { total, list, access:'free', available, message }
 */
export async function capabilitiesList(data: { filter?: Record<string, unknown>; page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/capabilitiesList',
      data,
    },
  })
  return { data: result }
}

export { requestBridge, request }
