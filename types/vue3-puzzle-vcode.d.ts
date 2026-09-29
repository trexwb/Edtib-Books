/**
 * vue3-puzzle-vcode 模块类型声明
 * 说明：该包（v1.1.7）的 package.json exports 未正确指向类型文件，
 * moduleResolution=bundler 模式下无法自动解析，故在此本地声明。
 */
declare module 'vue3-puzzle-vcode' {
  import type { DefineComponent } from 'vue'

  const VabSliderVerify: DefineComponent<{
    /** 拼图验证成功回调 */
    onSuccess?: (payload: unknown) => void
    /** 拼图验证失败回调 */
    onFail?: () => void
    /** 请求新的验证图 */
    refresh?: () => void
    /** 滑块验证模式（手动） */
    mode?: 'auto' | 'manual'
    /** 图片资源地址 */
    imgs?: string[]
    [key: string]: unknown
  }>

  export default VabSliderVerify
}
