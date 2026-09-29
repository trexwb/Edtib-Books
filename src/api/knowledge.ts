/**
 * @description 知识库资料文档 API（积分购买 + 已购下载）
 *
 * 对应服务端 FastenerKnowledgeService：
 * - list / detail：目录与详情（含 owned 已购标记）
 * - purchase：积分直扣购买（事务：扣款 + 流水 + user_knowledge 已购记录），幂等——已购直接报错引导下载
 * - owned：我的已购文档
 * - downloadUrl：已购文档下载地址（docs.path 走 /storage 静态通道）
 *
 * 扣费与已购记录均由服务端维护，离线 IPC 本地库不同步钱包，故全部走 HTTP 通道。
 *
 * @Author: trexwb
 * @Date: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 资料文档列表（分页）
 * HTTP: /front/knowledge/list
 * @param data.keyword 关键词（标题 / 分类模糊匹配）
 * @param data.category 分类
 * @returns { list, total, page, pageSize, categories }
 */
export async function knowledgeList(data: { keyword?: string; category?: string; page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/knowledge/list',
      data,
    },
  })
  return { data: result }
}

/**
 * 资料文档详情（含 owned 已购标记与 price_credit 所需积分）
 * HTTP: /front/knowledge/detail
 *   400600010001 知识库文档不存在
 */
export async function knowledgeDetail(data: { item_id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/knowledge/detail',
      data,
    },
  })
  return { data: result }
}

/**
 * 积分购买资料文档（幂等：已购不重复扣费）
 * HTTP: /front/knowledge/purchase
 * @returns { item_id, use_credits, balance, doc_id }
 *   400600010001 不存在 / 400600010002 已购买过该文档 / 400400010002 积分余额不足
 */
export async function knowledgePurchase(data: { item_id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/knowledge/purchase',
      data,
    },
  })
  return { data: result }
}

/**
 * 我的已购资料文档
 * HTTP: /front/knowledge/owned
 * @returns { list, total }
 */
export async function knowledgeOwned() {
  const result = await requestBridge.create({
    http: {
      url: '/front/knowledge/owned',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 已购资料文档下载地址
 * HTTP: /front/knowledge/downloadUrl
 * @returns { url, times_expire, file_name }
 *   400600010001 不存在 / 403600010003 尚未购买该文档
 */
export async function knowledgeDownloadUrl(data: { item_id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/knowledge/downloadUrl',
      data,
    },
  })
  return { data: result }
}

export { requestBridge }
