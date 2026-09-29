import progressModule from 'vite-plugin-vitebar'

// [迁移调整] vite-plugin-vitebar 是 tsup 产出的 CJS 包（module.exports.default）。
// 老项目是 CJS 工程，打包工具按 __esModule 约定取到 default 导出；本项目 package.json
// 带 "type": "module"，vite.config.ts 以 ESM 方式加载且该依赖被外部化，Node 对 CJS 的
// default 绑定等于 module.exports 本身，因此这里做一次兼容取值。
const progress = ((progressModule as any)?.default ?? progressModule) as typeof progressModule

export const createProgress = (env: Record<string, string>) => {
  return progress({ env })
}
