/**
 * @description router全局配置，如有必要可分文件抽离，其中asyncRoutes只有在intelligence模式下才会用到，
 * pro版只支持remixIcon图标，具体配置请查看vip群文档
 */
import type { App } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { createRouter, createWebHashHistory, createWebHistory } from 'vue-router'
import type { VabRouteRecordRaw } from './types'
import { authentication, base, isHashRouterMode } from '/@/config'
import { setupPermissions } from '/@/router/permissions'
import Layout from '/@vab/layouts/index.vue'

export const constantRoutes: VabRouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('/@/views/login/index.vue'),
    meta: {
      hidden: true,
    },
  },
  {
    path: '/redirect',
    name: 'Redirect',
    component: () => import('/@/views/redirect/Redirect.vue'),
    meta: {
      hidden: true,
    },
  },
  {
    path: '/403',
    name: '403',
    component: () => import('/@/views/error/403.vue'),
    meta: {
      hidden: true,
    },
  },
  {
    path: '/404',
    name: '404',
    component: () => import('/@/views/error/404.vue'),
    meta: {
      hidden: true,
    },
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Layout,
    meta: {
      title: '配置',
      icon: 'user-settings-line',
      hidden: true,
    },
    children: [
      {
        path: 'personalCenter',
        name: 'PersonalCenter',
        component: () => import('/@/views/settings/PersonalCenter.vue'),
        meta: {
          title: '个人中心',
          icon: 'map-pin-user-line',
        },
      },
    ],
  },
  // {
  //   path: '/standards',
  //   name: 'StandardsRoot',
  //   component: Layout,
  //   meta: {
  //     hidden: true,
  //   },
  //   children: [
  //     {
  //       path: 'products',
  //       name: 'StandardsProducts',
  //       component: () => import('/@/views/standards/products/index.vue'),
  //       meta: {
  //         hidden: true,
  //       },
  //       children: [
  //         {
  //           path: 'annotationInterpretation',
  //           name: 'AnnotationInterpretation',
  //           component: () => import('/@/views/standards/products/annotationInterpretation.vue'),
  //           meta: {
  //             hidden: true,
  //           },
  //         },
  //       ],
  //     }
  //   ],
  // }
]
export const asyncRoutes: VabRouteRecordRaw[] = [
  {
    path: '/',
    name: 'Root',
    component: Layout,
    meta: {
      title: '首页',
      icon: 'home-2-line',
      hidden: false,
      levelHidden: true,
      breadcrumbHidden: true,
    },
    children: [],
  },
]

const router = createRouter({
  // [迁移调整] hash 模式下不再传入 base：Tauri 使用 tauri://localhost 自定义协议，
  // createWebHashHistory 传入非空 base 会导致 hash 前的路径段与实际协议路径不一致
  history: isHashRouterMode ? createWebHashHistory() : createWebHistory(base),
  routes: constantRoutes as RouteRecordRaw[],
})

const fatteningRoutes = (routes: VabRouteRecordRaw[]): VabRouteRecordRaw[] => {
  return routes.flatMap((route) => {
    return route.children ? fatteningRoutes(route.children) : route
  })
}

const addRouter = (routes: VabRouteRecordRaw[]) => {
  routes.forEach((route: VabRouteRecordRaw) => {
    if (!router.hasRoute(route.name)) router.addRoute(route as RouteRecordRaw)
    if (route.children) addRouter(route.children)
  })
}

export const resetRouter = (routes: VabRouteRecordRaw[] = constantRoutes) => {
  routes.map((route: VabRouteRecordRaw) => {
    if (route.children) route.children = fatteningRoutes(route.children)
  })
  router.getRoutes().forEach((route: any) => {
    if (route.name) {
      const routeName: any = route.name
      router.hasRoute(routeName) && router.removeRoute(routeName)
    }
  })
  addRouter(routes)
}

export const setupRouter = (app: App<Element>) => {
  if (authentication === 'all') addRouter(asyncRoutes)
  setupPermissions(router)
  app.use(router)
  return router
}

export default router
