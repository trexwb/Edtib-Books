/**
 * vite-plugin-svg-icons 虚拟模块类型声明
 * 说明：该包（v2.2.1）未在 exports 中声明 ./client 子路径，
 * moduleResolution=bundler 模式下无法通过 types 引用，故在此本地声明。
 * 内容与 node_modules/vite-plugin-svg-icons/client.d.ts 保持一致。
 */
declare module 'virtual:svg-icons-register' {
  const component: any
  export default component
}

declare module 'virtual:svg-icons-names' {
  const iconsNames: string[]
  export default iconsNames
}
