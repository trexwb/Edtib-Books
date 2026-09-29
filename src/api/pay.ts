/**
 * @description 支付订单 API（HTTP 通道）
 * 二维码取码 / 订单状态查询 / 关闭订单；订单创建由 creditsPurchase / subscriptionSubscribe 下发
 *
 * @Author: trexwb
 * @Date: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'

/**
 * 订单支付状态查询（客户端 2s 轮询；服务端 8s 节流查渠道并自愈补结算）
 * HTTP: /front/pay/queryOrder
 * @returns { status: 'waiting' | 'paid' | 'closed' | 'expired', paid_at }
 *   400500010003 订单不存在
 */
export async function payQueryOrder(data: { order_no: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/pay/queryOrder',
      data,
    },
  })
  return { data: result }
}

/**
 * 关闭订单（待支付订单取消；渠道侧已支付则幂等补结算返回 paid）
 * HTTP: /front/pay/closeOrder
 * @returns { status }
 */
export async function payCloseOrder(data: { order_no: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/pay/closeOrder',
      data,
    },
  })
  return { data: result }
}

/**
 * 重取微信支付二维码（仅待支付订单）
 * HTTP: /front/pay/wechatQr
 * @returns { order_no, amount, qr_content, expire_at }
 */
export async function payWechatQr(data: { order_no: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/pay/wechatQr',
      data,
    },
  })
  return { data: result }
}

/**
 * 重取支付宝二维码（仅待支付订单）
 * HTTP: /front/pay/alipayQr
 * @returns { order_no, amount, qr_content, expire_at }
 */
export async function payAlipayQr(data: { order_no: string }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/pay/alipayQr',
      data,
    },
  })
  return { data: result }
}
