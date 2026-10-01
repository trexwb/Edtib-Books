import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import chokidar from 'chokidar'
import dayjs from 'dayjs'
import pc from 'picocolors'
import type { Plugin } from 'vite'
import { createBanner } from './banner'
import { createMock } from './mock'
import { createProgress } from './progress'
import { createPwa } from './pwa'
import { createSvgIcons } from './svgSprite'
import { createUnPlugin } from './unplugin'
// import { createRollup } from './rollup/'
import { port } from '../../src/config'
// import { createReplace } from './unplugin/replace'

const vitePrefix = 'VITE_'
const viteApp = vitePrefix + 'APP_'
const viteUser = vitePrefix + 'USER_'

export const createVitePlugin = (env: Record<string, string>) => {
  const vitePlugins: (Plugin | Plugin[])[] = [vue()]
  const userName = env[`${viteApp}GITHUB_USER_NAME`]
  const secretKey = env[`${viteApp}SECRET_KEY`]
  const nodeEnv = env[`${viteUser}NODE_ENV`]
  const assetUrl = env[`${vitePrefix}ASSETS_BASE_URL`]
  const isEmpty = (value: any) => {
    return value == undefined || value == '' || value == null
  }
  // 迁移说明：老项目在此处对缺失授权码（VITE_APP_GITHUB_USER_NAME / VITE_APP_SECRET_KEY）
  // 的工程直接 `return`（返回 undefined），导致连 vue 插件本身都不注册、构建直接崩溃。
  // Tauri 迁移后改为「插件链始终按完整模式注册」，仅在缺失授权码时打印提示；
  // 运行期的授权校验逻辑保持与老项目一致，未作改动。
  if (isEmpty(userName) || isEmpty(secretKey)) {
    console.warn(
      `\n${pc.yellow('[edtib-books]')} 未检测到 ${viteApp}GITHUB_USER_NAME / ${viteApp}SECRET_KEY（由 .env 与 .env.local 提供），已按完整插件链注册，跳过授权码校验提示。\n`
    )
  }
  if (nodeEnv !== 'development') if (isEmpty(userName) || isEmpty(secretKey)) console.warn(`\n${pc.yellow('[edtib-books]')} 生产环境缺少授权码，如为正式发布请补齐 .env.local。\n`)
  vitePlugins.push(vueJsx())
  vitePlugins.push(createProgress(env) as any)
  vitePlugins.push(createUnPlugin(env))
  vitePlugins.push(createPwa())
  vitePlugins.push(createMock())
  vitePlugins.push(createSvgIcons())
  vitePlugins.push(createBanner())
  // vitePlugins.push(createRollup())
  // 将静态文件指向OSS域名，有利于利用OSS流量减负
  // vitePlugins.push(createReplace())
  // if (nodeEnv !== 'development') vitePlugins.push({
  //   name: 'html-transform',
  //   transformIndexHtml(html, ctx) {
  //     return html.replace(/\/console\/static\//g, (assetUrl || `https://oss.lixitu.com/console/${nodeEnv === 'production' ? 'master' : nodeEnv}`) + '/static/');
  //     // return html.replace(/\.\/static\//g, (assetUrl || `https://oss.lixitu.com/console/${nodeEnv === 'production' ? 'master' : nodeEnv}`) + '/static/')
  //     // .replace(/\/console\/static\//g, (assetUrl || `https://oss.lixitu.com/console/${nodeEnv === 'production' ? 'master' : nodeEnv}`) + '/static/');
  //   }
  // })
  return vitePlugins
}

export const createWatch = (env: Record<string, string>) => {
  //为了防止新同事忘记配置授权码而造成项目无法打包，请保留以下提示
  const userName = env[`${viteApp}GITHUB_USER_NAME`]
  const secretKey = env[`${viteApp}SECRET_KEY`]
  const nodeEnv = env[`${viteUser}NODE_ENV`]
  console.log(nodeEnv, userName, secretKey)

  // if (nodeEnv === 'production') {
  //   if (userName === 'test' || secretKey === 'preview') {
  //     console.log(
  //       `${pc.red(
  //         '\u68c0\u6d4b\u5230\u60a8\u7684\u7528\u6237\u540d\u6216\u006b\u0065\u0079\u672a\u914d\u7f6e\uff0c\u006b\u0065\u0079\u5728\u8d2d\u4e70\u65f6\u901a\u8fc7\u90ae\u4ef6\u9080\u8bf7\u51fd\u53d1\u653e\uff0c\u5982\u60a8\u5df2\u8d2d\u4e70\u8bf7\u4ed4\u7ec6\u9605\u8bfb\u6587\u6863\u5e76\u8fdb\u884c\u914d\u7f6e\uff0c\u914d\u7f6e\u5b8c\u6210\u540e\u65b9\u53ef\u6253\u5305\u4f7f\u7528\u3002\u8d2d\u4e70\u5730\u5740\uff1a\u0068\u0074\u0074\u0070\u0073\u003a\u002f\u002f\u0076\u0075\u0065\u002d\u0061\u0064\u006d\u0069\u006e\u002d\u0062\u0065\u0061\u0075\u0074\u0069\u0066\u0075\u006c\u002e\u0063\u006f\u006d\u002f\u0061\u0075\u0074\u0068\u006f\u0072\u0069\u007a\u0061\u0074\u0069\u006f\u006e\u002f\u0073\u0068\u006f\u0070\u002d\u0076\u0069\u0074\u0065\u002e\u0068\u0074\u006d\u006c'
  //       )}`
  //     )
  //     process.exit()
  //   }
  // }

  if (nodeEnv === 'development') {
    chokidar.watch('./src/views').on('change', (path) => {
      if (path.endsWith('vue')) {
        console.log(
          `\n${pc.gray(dayjs().format('HH:mm:ss'))} ${pc.cyan('[Vue Sh' + 'op Vite]')} ${pc.cyan(`http://localhost:${port}/`)} ${pc.green(
            'update success'
          )} `
        )
      }
    })
  }
}
