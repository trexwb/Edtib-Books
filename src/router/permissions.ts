/**
 * @description 路由守卫，目前两种模式：all模式与intelligence模式
 */
import VabProgress from 'nprogress'
import 'nprogress/nprogress.css'
import { Router } from 'vue-router'
import { authentication, loginInterception, routesWhiteList, supportVisit } from '/@/config'
import { useRoutesStore } from '/@/store/modules/routes'
import { useSettingsStore } from '/@/store/modules/settings'
import { useUserStore } from '/@/store/modules/user'
import getPageTitle from '/@/utils/pageTitle'
import { toLoginRoute } from '/@/utils/routes'

export function setupPermissions(router: Router) {
  VabProgress.configure({
    easing: 'ease',
    speed: 500,
    trickleSpeed: 200,
    showSpinner: false,
  })
  router.beforeEach(async (to, from, next) => {
    const {
      getTheme: { showProgressBar },
    } = useSettingsStore()
    const { routes, setRoutes } = useRoutesStore()
    const { token, getUserInfo, setVirtualRoles, resetAll } = useUserStore()

    if (showProgressBar) VabProgress.start()

    let hasToken: boolean = !!token

    if (!loginInterception) hasToken = true

    if (hasToken) {
      if (routes.length) {
        // 禁止已登录用户返回登录页
        if (to.path === '/login') {
          next({ path: '/' })
          if (showProgressBar) VabProgress.done()
        } else next()
      } else {
        try {
          if (loginInterception) await getUserInfo()
          // config/setting.config.js loginInterception为false(关闭登录拦截时)时，创建虚拟角色
          else await setVirtualRoles()
          // 根据路由模式获取路由并根据权限过滤
          await setRoutes(authentication)
          next({ ...to, replace: true })
        } catch (err: any) {
          console.error('错误拦截:', err)
          // P1-5 修复：仅 token 无效（401/402 业务码）才清除登录态重置路由；
          // 网络抖动/后端不可达等瞬时故障保留 token，不清凭据，仅送回登录页等待重试
          const bizCode = Number(err?.code ?? NaN)
          const isAuthError = bizCode === 401 || bizCode === 402
          if (isAuthError) await resetAll()
          next(toLoginRoute(to.path))
        }
      }
    } else {
      if (routesWhiteList.includes(to.path)) {
        // 设置游客路由(不需要可以删除)
        if (supportVisit && !routes.length) {
          await setRoutes('visit')
          next({ path: to.path, replace: true })
        } else next()
      } else next(toLoginRoute(to.path))
    }
  })
  router.afterEach((to) => {
    if (typeof to.meta.title === 'string') document.title = getPageTitle(to.meta.title)
    if (VabProgress.status) VabProgress.done()
  })
}
