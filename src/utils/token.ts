import { storage as _storage, tokenTableName } from '/@/config'

const storage = _storage || 'localStorage'

/**
 * @description 获取token
 */
export const getToken = () => {
  if ('localStorage' === storage) return localStorage.getItem(tokenTableName)
  else if ('sessionStorage' === storage) return sessionStorage.getItem(tokenTableName)
}

/**
 * @description 存储token
 * @param token
 */
export const setToken = (token: string) => {
  if ('localStorage' === storage) return localStorage.setItem(tokenTableName, token)
  else if ('sessionStorage' === storage) return sessionStorage.setItem(tokenTableName, token)
}

/**
 * @description 移除token（P2-2 修复：默认读统一 storage 配置，原硬编码 'localStorage'
 *               遮蔽了模块顶部的 storage 变量，切 sessionStorage 时将清不掉 token；
 *               sessionStorage 分支只移除 token 键，不再 clear() 整个存储）
 */
export const removeToken = (storageOverride?: string) => {
  const target = storageOverride || storage
  if ('localStorage' === target) return localStorage.removeItem(tokenTableName)
  else if ('sessionStorage' === target) return sessionStorage.removeItem(tokenTableName)
}
