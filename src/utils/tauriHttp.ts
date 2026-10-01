/**
 * @description Tauri 环境统一 HTTP 适配器（P0-2 修复）
 *
 * 迁移要求「HTTP 统一在 Rust 侧发起」：Tauri 生产包从 tauri://localhost 加载，
 * axios 的相对路径 baseURL（/api）没有 dev server 代理可解析，必然失败。
 * 本适配器把 axios 请求转译为 invoke('http_request')，由 Rust reqwest 栈发起，
 * 并在客户端补齐网关契约要求的签名 / 加密头：
 *   * App-Id / App-Nonce / App-Secret（AuthenticateSecret 校验，nonce 一次性）；
 *   * X-Sign（VerifySignature 校验，VERIFY_SIGNATURE=true 时必填，
 *     对「最终 body（含加密后的 encryptedData）/ query」排序序列化后计算）；
 *   * 请求加密体 encryptedData（网关 Crypto 已升级为「随机 IV 前缀 iv:cipher」格式，
 *     客户端同步使用随机 IV 并前缀传输）。
 *
 * 响应侧：Rust 返回 { status, ok, headers, contentType, data }，
 * data 已按网关信封 { code, data, msg } 解析；encryptedData 仍交由 handleData 解密，
 * 以复用既有的信封 code 分支 / 401 登出 / 错误提示逻辑。
 * @Author: trexwb
 */

import { invoke } from '@tauri-apps/api/core'
import { stringify } from 'qs'
import CryptoJS from 'crypto-js'
import { contentType as defaultContentType } from '/@/config'

/** 与 src/utils/request.ts 保持一致的随机串生成（升级为 crypto.getRandomValues，CSPRNG） */
const generateRandomString = (length: number): string => {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)
  let result = ''
  for (let i = 0; i < length; i++) {
    result += characters.charAt(bytes[i] % charactersLengthSafe(characters.length))
  }
  return result
}
const charactersLengthSafe = (total: number): number => total

const md5 = (str: string): string => CryptoJS.MD5(str).toString()
const sha256 = (str: string): string => CryptoJS.SHA256(str).toString()

/** 网关 DecryptRequest/Crypto 契约：密钥 32 字节、IV 16 字节（UTF-8 长度） */
const encrypt = (data: any, key: string): string => {
  if (!key || key.length !== 32) throw new Error('Invalid key length')
  const ivWordArray = CryptoJS.lib.WordArray.random(16)
  const keyWordArray = CryptoJS.enc.Utf8.parse(key)
  const encryptedText = JSON.stringify(data)
  const encrypted = CryptoJS.AES.encrypt(encryptedText, keyWordArray, { iv: ivWordArray }).ciphertext.toString(CryptoJS.enc.Hex)
  // 网关新格式（UT-01）：iv(hex) + ':' + cipher(hex)
  return `${ivWordArray.toString(CryptoJS.enc.Hex)}:${encrypted}`
}

/** 与网关 VerifySignature#sortObjectDeep 一致的深度排序序列化 */
const sortObjectDeep = (value: any): any => {
  if (Array.isArray(value)) return value.map(sortObjectDeep)
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([k, v]) => [k, sortObjectDeep(v)])
    )
  }
  return value
}

/** X-Sign：md5( sha256(JSON.stringify(排序后参数)) + appKey ) */
export const computeXSign = (params: Record<string, any>, appKey: string): string => {
  const sorted = sortObjectDeep(params ?? {})
  return md5(sha256(JSON.stringify(sorted)) + appKey)
}

export interface TauriHttpRequestConfig {
  url: string
  method?: string
  data?: any
  params?: any
  headers?: Record<string, any>
  timeout?: number
}

/**
 * 发起一次经 Rust 侧转发的 HTTP 请求。
 *
 * 返回值模拟 axios response 的最小消费面（request.ts#handleData 只用到
 * headers / config / data / status / statusText 五个字段）。
 */
export const tauriHttpRequest = async (config: TauriHttpRequestConfig): Promise<any> => {
  const appId = import.meta.env.VITE_APP_ID as string
  const appSecret = import.meta.env.VITE_APP_SECRET as string
  const requestEncrypt = import.meta.env.VITE_REQUEST_ENCRYPT === 'true'

  const method = (config.method || 'get').toUpperCase()
  const headers: Record<string, string> = {
    'Content-Type': defaultContentType,
    ...(config.headers || {}),
  }
  // 清理 undefined / null 值的头
  Object.keys(headers).forEach((name) => {
    if (headers[name] === undefined || headers[name] === null) delete headers[name]
  })

  // 1) 网关签名三件套（AuthenticateSecret：nonce 一次性，服务端 Redis 去重）
  const timeStamp = Math.floor(Date.now() / 1000).toString()
  const nonce = generateRandomString(32)
  headers['App-Id'] = `${appId}`
  headers['App-Nonce'] = nonce
  headers['App-Secret'] = md5(`${sha256(`${appId}${timeStamp}${nonce}`)}${appSecret}`) + timeStamp

  // 2) 请求体：加密（随机 IV 前缀）或表单序列化，得到「最终 body」
  // Rust 侧（http.rs）对字符串按原文发送、对对象按 application/json 序列化，
  // 因此表单场景必须在此序列化为 urlencoded 字符串，否则实发 JSON 与声明的 Content-Type 不符。
  let finalBody: Record<string, any> | string | undefined
  if (config.data !== undefined) {
    if (requestEncrypt && config.data) {
      finalBody = { encryptedData: encrypt(config.data, appSecret) }
    } else if (headers['Content-Type'] === 'application/x-www-form-urlencoded;charset=UTF-8') {
      // 表单场景：网关侧 body 是序列化字符串，签名按服务端解析后的 {key:value} 结构
      finalBody = typeof config.data === 'string' ? config.data : stringify(config.data)
    } else {
      finalBody = config.data
    }
  }

  // 3) X-Sign（VerifySignature：对 { ...query, ...最终 body } 签名，必填）
  // 表单场景签名对象是序列化前的键值对象（Express urlencoded 解析后即该结构）
  const signSource: Record<string, any> = typeof finalBody === 'string' ? (config.data ?? {}) : (finalBody ?? {})
  const signParams: Record<string, any> = { ...(config.params || {}), ...signSource }
  headers['X-Sign'] = computeXSign(signParams, appSecret)

  // 4) URL / 查询串
  let url = config.url
  const queryString = config.params ? stringify(config.params) : ''
  if (queryString) url = `${url}${url.includes('?') ? '&' : '?'}${queryString}`

  // 5) 交由 Rust reqwest 栈发起（仅支持 http/https，命令侧已校验）
  const payload = await invoke<{
    status: number
    ok: boolean
    headers: Record<string, string>
    contentType: string | null
    data: any
  }>('http_request', {
    url,
    method,
    headers,
    body: finalBody === undefined ? null : JSON.parse(JSON.stringify(finalBody)),
  })

  // 6) 还原为 handleData 期望的 response 形态
  return {
    headers: payload.headers || {},
    status: payload.status,
    statusText: payload.ok ? 'OK' : 'Error',
    config: { ...config, url },
    data: payload.data,
  }
}

/** 当前是否运行在 Tauri 环境（与 src/bridge 判定一致，避免循环依赖此处自行判定） */
export const isTauriRuntime = (): boolean => {
  if (typeof window === 'undefined') return false
  const w = window as any
  return Boolean(w.__TAURI_INTERNALS__ || w.__TAURI__)
}
