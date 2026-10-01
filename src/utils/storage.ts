/*
 * @Author: trexwb
 * @Date: 2024-01-27 09:13:55
 * @LastEditTime: 2026-09-10 17:02:55
 * @LastEditors: ${git_name}
 * @Description: In User Settings Edit
 * @FilePath: /fastenerTradeWorkbench/Users/wbtrex/website/localServer/node/edtib/client/books/web/src/utils/storage.ts
 * 一花一世界，一叶一如来
 * Copyright (c) 2024 by 杭州大美科技, All Rights Reserved. 
 */
import { isJson } from '/@/utils/validate'

const prefix = 'book'

/**
 * @description 获取localStorage
 */
export const getStorage = (key: string) => {
  const value: any = localStorage.getItem(`${prefix}_${key}`)
  // 仅当确实拿到值且值为合法 JSON 时才反序列化；
  // 原 `value || isJson(value)` 对任意非空字符串都会 JSON.parse，直接抛 SyntaxError
  if (value && isJson(value)) {
    return JSON.parse(value)
  } else {
    return value
  }
}

/**
 * @description 存储localStorage
 * @param key
 * @param value
 */
export const setStorage = (key: string, value: any) => {
  return localStorage.setItem(`${prefix}_${key}`, JSON.stringify(value))
}

/**
 * @description 存储localStorage
 * @param key
 * @param value
 */
export const removeStorage = (key: string) => {
  return localStorage.removeItem(`${prefix}_${key}`)
}