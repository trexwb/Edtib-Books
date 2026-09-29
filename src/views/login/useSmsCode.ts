/**
 * @description 短信验证码（登录 / 注册共用）
 * 职责：手机号校验 → 下发验证码 → 缓存 token → 60s 重发倒计时 → 失败兜底
 * 登录与注册仅 scene 不同：1=登录（手机号须已注册），2=注册
 * @Author: trexwb
 * @Date: 2026-09-10
 */

import { onBeforeUnmount, reactive } from 'vue'

import { sendSmsCode } from '/@/api/authorize'
import { translate } from '/@/i18n'
import { isPhone } from '/@/utils/validate'

/** 与后端 SmsScene 对齐 */
export const SMS_SCENE = {
  LOGIN: 1,
  REGISTER: 2,
} as const

export type SmsScene = (typeof SMS_SCENE)[keyof typeof SMS_SCENE]

/**
 * 验证码 token 已作废的错误码（后端 SmsErrorCode）
 * CODE_EXPIRED / CODE_ATTEMPTS_EXCEEDED / CODE_MOBILE_MISMATCH
 * 不含 CODE_INVALID——此时 token 仍有效（还可尝试 N 次），保留以便用户重试、省一次短信
 */
const SMS_TOKEN_INVALID_CODES = [400030007009, 400030007011, 400030007012]

type SmsMessage = (message: string, type: 'success' | 'warning' | 'error', icon?: string) => void

export const useSmsCode = (scene: SmsScene, options: { message: SmsMessage }) => {
  const state = reactive({
    /** 验证码下发中（按钮 loading） */
    sending: false,
    /** 重发倒计时剩余秒数 */
    countdown: 0,
    /** sendSmsCode 返回的凭证，register / signSms 时回传 */
    token: '',
  })

  let timer: any = null

  const clearCountdown = () => {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  const startCountdown = (seconds = 60) => {
    clearCountdown()
    state.countdown = seconds
    timer = setInterval(() => {
      state.countdown -= 1
      if (state.countdown <= 0) clearCountdown()
    }, 1000)
  }

  /** 清空 token 与倒计时（更换手机号、登录/注册成功后调用） */
  const reset = () => {
    clearCountdown()
    state.countdown = 0
    state.token = ''
  }

  /** 发送验证码：校验手机号 → 下发 → 缓存 token → 进入重发倒计时 */
  const send = async (mobile: string) => {
    if (state.sending || state.countdown > 0) return
    if (!isPhone(mobile)) {
      options.message(translate('请输入正确的手机号'), 'warning', 'hey')
      return
    }
    state.sending = true
    try {
      const { data } = await sendSmsCode({ mobile, scene })
      state.token = data?.token || ''
      startCountdown(60)
      options.message(translate('验证码已发送，请注意查收'), 'success', 'hey')
    } catch {
      // 业务错误已由请求拦截器统一提示，此处仅兜底
      state.token = ''
    } finally {
      state.sending = false
    }
  }

  /** 提交环节失败：仅验证码已作废时才清 token，其余保留供用户重试 */
  const handleError = (error: any) => {
    if (SMS_TOKEN_INVALID_CODES.includes(Number(error?.code))) reset()
  }

  // 组件卸载时清理定时器
  onBeforeUnmount(clearCountdown)

  return Object.assign(state, { send, reset, handleError })
}
