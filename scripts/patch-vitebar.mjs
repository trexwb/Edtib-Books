// [迁移调整] 由老项目 web/install.js 的 postinstall 逻辑迁移而来。
// 老项目在依赖安装后会把 replace/vitebar_index.js 覆盖到
// node_modules/vite-plugin-vitebar/dist/index.js（本地定制版打包进度条插件，
// 需要替换上游实现以适配本项目）。Tauri 项目保持同样的安装后处理，
// 保证依赖被重装后该定制不会丢失。
//
// 另外补充一处迁移适配：vite-plugin-vitebar@0.0.8 的 package.json 只声明了
// exports["."].require，而本项目 package.json 带 "type": "module"，Vite 在打包
// vite.config.ts 时按 ESM 条件解析依赖，会报
// "No known conditions for '.' specifier in vite-plugin-vitebar package"。
// 这里为它补一个指向同一产物的 "import" 条件（老项目 package.json 无 "type": "module"，
// 走 require 条件，因此不存在该问题）。
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const source = resolve(root, 'replace/vitebar_index.js')
const distTarget = resolve(root, 'node_modules/vite-plugin-vitebar/dist/index.js')
const pkgTarget = resolve(root, 'node_modules/vite-plugin-vitebar/package.json')

if (!existsSync(source)) {
  console.warn(`[patch-vitebar] 跳过：未找到 ${source}`)
} else if (!existsSync(distTarget)) {
  console.warn(`[patch-vitebar] 跳过：未找到 ${distTarget}（请先安装依赖）`)
} else {
  copyFileSync(source, distTarget)
  console.log('[patch-vitebar] 已应用 vite-plugin-vitebar 定制实现')
}

if (existsSync(pkgTarget)) {
  const pkg = JSON.parse(readFileSync(pkgTarget, 'utf8'))
  const dot = pkg.exports && pkg.exports['.']
  if (dot && typeof dot === 'object' && !dot.import) {
    dot.import = dot.require || './dist/index.js'
    writeFileSync(pkgTarget, `${JSON.stringify(pkg, null, 2)}\n`)
    console.log('[patch-vitebar] 已为 vite-plugin-vitebar 补充 exports["."].import 条件')
  } else {
    console.log('[patch-vitebar] vite-plugin-vitebar exports 条件已满足，无需处理')
  }
}
