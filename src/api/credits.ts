/**
 * @description 积分中心 API（积分余额 / 套餐充值 / 消费流水 / 兑换码）
 *
 * 积分钱包（credit_wallets）与流水（credit_transactions）由服务端事务维护：
 * 变更唯一入口为 FastenerCreditsService.changeBalance（行锁 + 流水 + customer_users.credit 同步），
 * 离线 IPC 本地库不存在钱包与流水，若在本地写入会与服务端余额冲突，故本模块全部走 HTTP 通道。
 *
 * @Author: trexwb
 * @Date: 2026-04-15 23:50
 * @LastEditTime: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 积分余额（含累计充值 / 累计消费）
 * HTTP: /front/credits/balance
 * @returns { balance, total_recharge, total_consume }
 */
export async function creditsBalance() {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/balance',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 积分充值套餐（服务端 DEFAULT_PACKAGES，可被环境变量 FASTENER_CREDIT_PACKAGES 覆盖）
 * HTTP: /front/credits/packages
 * @returns { list: [{ id, credits, price, tag }] }，price 单位为分
 */
export async function creditsPackages() {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/packages',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 充值下单（生成待支付订单，返回二维码渠道信息）
 * HTTP: /front/credits/purchase
 * @param data.package_id 套餐编号
 * @param data.channel 支付渠道：1 微信 / 2 支付宝
 * @returns 订单信息（order_no / amount / status / qr）
 *   400400010001 充值包不存在 / 400400010003 channel 仅支持 1 微信 / 2 支付宝
 */
export async function creditsPurchase(data: { package_id: number; channel?: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/purchase',
      data: { channel: 1, ...data },
    },
  })
  return { data: result }
}

/**
 * 积分消费流水（分页，倒序）
 * HTTP: /front/credits/transactions
 * @returns { list, total, page, pageSize }
 */
export async function creditsTransactions(data: { page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/transactions',
      data,
    },
  })
  return { data: result }
}

/**
 * 我的订单列表（订阅 / 积分充值，分页倒序）
 * HTTP: /front/credits/orders
 * @returns { list, total, page, pageSize }
 *   list 项含 order_no / scene(1订阅 2积分充值) / amount(分) / channel(1微信 2支付宝)
 *   / status(0待支付 1已支付 2已关闭 3已退款) / paid_at / times_expire / created_at / extension
 */
export async function creditsOrders(data: { page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/orders',
      data,
    },
  })
  return { data: result }
}

/**
 * 兑换码兑换（积分码；订阅时长码请到订阅页兑换）
 * HTTP: /front/credits/redeem
 * @returns { credits, balance }
 *   400400010004 兑换码不存在 / 400400010005 已被使用或作废 / 400400010006 该码为订阅时长码
 */
export async function creditsRedeem(data: { code: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/credits/redeem',
      data,
    },
  })
  return { data: result }
}

export { requestBridge, request }