import App from './App.vue'

import ElementPlus from 'element-plus'
import ElementPlusLocaleZhCn from 'element-plus/es/locale/lang/zh-cn'
import VXETable from 'vxe-table'
import { setupVab } from '~/library'
import { setupI18n } from '/@/i18n'
import { setupRouter } from '/@/router'
import { setupStore } from '/@/store'
import { themeConfig } from '/@/config/theme.config'

import JsonSchemaEditor from 'json-schema-editor-vue3'
// 虚拟滚动组件（用于大列表优化）
import { RecycleScroller, DynamicScroller, DynamicScrollerItem } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

import { useSettingsStore } from '/@/store/modules/settings'
import { getStorage } from '/@/utils/storage'
// [迁移调整] Electron IPC -> Tauri invoke 桥接层（所有主进程能力调用点统一收敛于 src/bridge）
import { setupBridge } from '/@/bridge'

const settingsStore = useSettingsStore()
const { changeTitle, getTitle, changeLogo } = settingsStore

const changeFavicon = (icon: string) => {
  let $favicon = document.querySelector('link[rel="icon"]') as HTMLLinkElement | null
  if ($favicon !== null) {
    $favicon.href = icon
  } else {
    $favicon = document.createElement('link')
    $favicon.rel = 'icon'
    $favicon.href = icon
    document.head.appendChild($favicon)
  }
}


// 动态设置 CSS 变量
const setGlobalCSSVariable = () => {
  const root = document.documentElement; // 获取 <html> 元素
  const colorItem = localStorage.getItem('color'); // 从 localStorage 中获取颜色
  let savedColor: string | null = null;

  if (colorItem) {
    try {
      const parsedColor = JSON.parse(colorItem);
      savedColor = parsedColor.color || themeConfig.color; // 提取颜色值并提供默认值
    } catch (error) {
      console.error('解析 color 数据时出错:', error);
      savedColor = themeConfig.color; // 如果解析失败，使用默认颜色
    }
  } else {
    savedColor = themeConfig.color; // 如果没有存储的颜色，使用默认颜色
  }

  if (savedColor) {
    root.style.setProperty('--default-color', savedColor); // 设置 CSS 变量
  }
};

// 在应用启动时调用
setGlobalCSSVariable();
const siteConfig = getStorage('siteConfig') || {}
if (!siteConfig.id || !getTitle) {
} else {
  if (siteConfig.title) (document.title = siteConfig.title), changeTitle(siteConfig.title)
  if (siteConfig.logo) changeLogo(siteConfig.logo)
  if (siteConfig.icon) changeFavicon(siteConfig.icon)
}

const app = createApp(App)
// 注册虚拟滚动组件为全局组件
app.component('RecycleScroller', RecycleScroller)
app.component('DynamicScroller', DynamicScroller)
app.component('DynamicScrollerItem', DynamicScrollerItem)
app.use(ElementPlus, { locale: ElementPlusLocaleZhCn }).use(VXETable).use(JsonSchemaEditor)

// [迁移调整] 安装 Electron IPC -> Tauri invoke 兼容层（幂等；非 Tauri 环境为空操作）
setupBridge()
setupVab(app)
setupI18n(app)
setupStore(app)
setupRouter(app)
  .isReady()
  .then(() => app.mount('#app'))
