/**
 * @description 订阅 API（订阅状态 / 套餐 / 下单 / 时长兑换码）
 *
 * 订阅权益：全部产品标准 + 当年标准更新；积分单独购买的标准保留购买时版本、不随更新同步。
 * 订单支付链路与积分充值一致（下单返回二维码内容，轮询 /front/pay/queryOrder）。
 *
 * @Author: trexwb
 * @Date: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 订阅套餐列表
 * HTTP: /front/subscription/plans
 * @returns { list: [{ id, names, price, original_price, duration_days, gift_credits, description, status, sort }] }
 *   price 单位为分；duration_days 订阅时长（天）；gift_credits 订阅赠送积分
 */
export async function subscriptionPlans() {
  const result = await requestBridge.create({
    http: {
      url: '/front/subscription/plans',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 订阅下单（生成待支付订单，返回二维码渠道信息）
 * HTTP: /front/subscription/subscribe
 * @param data.plan_id 套餐编号
 * @param data.channel 支付渠道：1 微信 / 2 支付宝
 * @returns { order_no, amount, qr_content, expire_at }
 *   400300010001 订阅计划不存在 / 400300010002 channel 仅支持 1 微信 / 2 支付宝
 */
export async function subscriptionSubscribe(data: { plan_id: number; channel?: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/subscription/subscribe',
      data: { channel: 1, ...data },
    },
  })
  return { data: result }
}

/**
 * 我的订阅状态
 * HTTP: /front/subscription/mySubscription
 * @returns { active, plan, start_at, end_at, days_left, status: 'none' | 'active' | 'expired' }
 */
export async function mySubscription() {
  const result = await requestBridge.create({
    http: {
      url: '/front/subscription/mySubscription',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 兑换订阅时长码（在现有到期时间上顺延）
 * HTTP: /front/subscription/redeem
 * @returns { type: 'subscription', value(天), end_at }
 *   400300010003 兑换码不存在 / 400300010004 已被使用或作废 / 400300010005 该码为积分码，请到积分页兑换
 */
export async function subscriptionRedeem(data: { code: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/subscription/redeem',
      data,
    },
  })
  return { data: result }
}
