/**
 * @description 授权认证 API（部分离线优先）
 * 登录操作必须在线验证，但用户信息可缓存到本地
 * @Author: trexwb
 * @Date: 2026-04-15 23:40
 */

import request from '/@/utils/request'
import requestBridge from '/@/utils/requestBridge'

import CryptoJS from 'crypto-js'

const md5 = (str: string) => {
  return CryptoJS.MD5(str).toString()
}

/**
 * 登录态落本地缓存
 * signIn / signSms / register / signSecret 共用：后端返回字段为 auth_token（非 token），仅 Electron（IPC）环境写盘
 */
const persistLoginCache = async (result: any) => {
  if (result?.data?.auth_token && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'user/login-cache.json', data: JSON.stringify(result.data) },
    })
  }
  return result
}

/**
 * 密码登录（必须在线）
 * IPC: 不适用
 * HTTP: /front/authorize/signIn
 */
export async function login(data: { username?: string; password?: string }) {
  data.password = md5(data.password || '')
  const result = await request({
    url: '/front/authorize/signIn',
    method: 'post',
    data,
  })
  
  // 登录成功后缓存用户信息
  await persistLoginCache(result)
  
  return result
}

/**
 * 发送短信验证码（必须在线）
 * IPC: 不适用
 * HTTP: /front/authorize/sendSmsCode
 * @param data.scene 1=登录场景（手机号须已注册，走 login 模板）；缺省=注册场景
 * @returns data: { token, expire_seconds }，token 需在 register / signSms 时回传
 */
export async function sendSmsCode(data: { mobile: string; scene?: number }) {
  return request({
    url: '/front/authorize/sendSmsCode',
    method: 'post',
    data,
  })
}

/**
 * 短信验证码登录（必须在线）
 * 手机号 + 短信验证码（sendSmsCode scene=1 返回的 token）登录，成功返回与 signIn 同构的登录态
 * IPC: 不适用
 * HTTP: /front/authorize/signSms
 */
export async function signSms(data: { mobile: string; code: string; token: string }) {
  const result = await request({
    url: '/front/authorize/signSms',
    method: 'post',
    data,
  })

  // 登录成功后缓存用户信息，与 login 保持一致
  await persistLoginCache(result)

  return result
}

/**
 * 手机号注册（必须在线）
 * 手机号 + 短信验证码 + 设置密码即完成注册，成功返回与 signIn 同构的登录态
 * IPC: 不适用
 * HTTP: /front/authorize/register
 */
export async function register(data: {
  mobile?: string
  token?: string
  code?: string
  password?: string
  nickname?: string
}) {
  data.password = md5(data.password || '')
  const result = await request({
    url: '/front/authorize/register',
    method: 'post',
    data,
  })

  // 注册成功即登录态：与 login 一致，缓存用户信息
  await persistLoginCache(result)

  return result
}

/**
 * 登录用户信息（离线优先）
 * IPC: fs.readFile（本地缓存）
 * HTTP: /front/authorize/signInfo
 */
export async function getUserInfo() {
  // 优先读取本地缓存
  const cached = await requestBridge.readFile({
    ipc: { filePath: 'user/login-cache.json' },
  })
  
  if (cached) {
    const cacheData = JSON.parse(cached)
    // 缓存存的是 signIn 返回结构（user 嵌套），需展平为 signInfo 的扁平结构
    if (cacheData?.user) {
      return {
        data: {
          ...cacheData.user,
          token: cacheData.auth_token,
          roles: cacheData.roles,
          permissions: cacheData.permissions,
          credit: cacheData.credit,
        },
      }
    }
  }
  
  // Fallback HTTP（获取最新用户信息）
  const { data } = await request({
    url: '/front/authorize/signInfo',
    method: 'post',
  })
  
  // 更新缓存
  if (data && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'user/login-cache.json', data: JSON.stringify(data) },
    })
  }
  
  return { data }
}

/**
 * 个人信息修改（部分离线）
 * IPC: 本地缓存更新
 * HTTP: /front/customers/usersSave
 */
export async function usersSave(data: {
  username?: string
  password?: string
  nickname?: string
  mobile?: string
  avatar?: string
  email?: string
}) {
  data.password = !data.password ? '' : md5(data.password)
  const result = await request({
    url: '/front/customers/usersSave',
    method: 'post',
    data,
  })
  
  // 成功后更新本地缓存
  if (result?.data && requestBridge.isElectron()) {
    const cached = await requestBridge.readFile({
      ipc: { filePath: 'user/login-cache.json' },
    })
    if (cached) {
      const cacheData = JSON.parse(cached)
      cacheData.user = { ...cacheData.user, ...data }
      await requestBridge.writeFile({
        ipc: { filePath: 'user/login-cache.json', data: JSON.stringify(cacheData) },
      })
    }
  }
  
  return result
}

/**
 * 退出登录
 * IPC: 清空本地缓存
 * HTTP: /front/authorize/logout
 */
export async function logout() {
  // 清空本地缓存
  if (requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: 'user/login-cache.json', data: '' },
    })
  }
  
  // 同时通知服务器
  return request({
    url: '/front/authorize/logout',
    method: 'post',
  })
}

/**
 * 密钥登录（必须在线）
 * IPC: 不适用
 * HTTP: front/authorize/signSecret
 */
export async function signSecret(data: { uuid?: string; secret?: string }) {
  data.secret = md5(`${data.uuid || ''}${data.secret || ''}`)
  const result = await request({
    url: '/front/authorize/signSecret',
    method: 'post',
    data,
  })
  
  // 登录成功后缓存
  await persistLoginCache(result)
  
  return result
}

export { requestBridge }