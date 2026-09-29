<!--
 * @Author: ${git_name}
 * @Date: 2025-04-02 14:36:05
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-08 09:41:17
 * @FilePath: /books/web/src/App.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-app />
</template>

<script lang="ts" setup>
import { useSettingsStore } from '/@/store/modules/settings'
// import { getStorage } from '/@/utils/storage'
// import Watermark from '/@/utils/watermark'
// import { useUserStore } from '/@/store/modules/user'

// import * as math from 'mathjs'
// let scope1 = { dMax: 10, dMin: 6, d: 5 };
// let result1 = math.evaluate("and(dMax, dMin) ? (dMax + dMin) / 2 : dMax ? dMax : dMin ? dMin : d", scope1);
// console.log(result1); // 8 ✅

defineOptions({
  name: 'App',
})

// 说明：Window.electronAPI 的完整类型声明统一维护在 src/types/electron.d.ts
// （此处曾有一份不完整的冗余声明，与 electron.d.ts 冲突导致 db/fs/system 类型丢失，已移除）

const $baseNotify = inject<any>('$baseNotify')
const settingsStore = useSettingsStore()
const { updateTheme, changeColor, handleLock } = settingsStore

// const { title } = storeToRefs(settingsStore)
// Watermark.set(title.value);
// Watermark.set('杭州仟标科技(edtib.com)');

// const userStore = useUserStore()
// const { avatar, username } = storeToRefs(userStore)
// Watermark.set(`${username.value || '游客'} - 杭州仟标`);

// const isBackground = ref(false)
// const lockTime = 5 * 60 // 秒为单位
// let lockTimer: any = null
// const startLockTimer = () => {
//   if (process.env.NODE_ENV === 'development') console.log('startLockTimer: ', isBackground.value, new Date())
//   const loginFormData = getStorage('loginFormData') || {}
//   if (isBackground.value && loginFormData['username']) {
//     lockTimer = setTimeout(() => {
//       handleLock()
//     }, lockTime * 1000)
//   }
// }
// const endLockTimer = () => {
//   if (process.env.NODE_ENV === 'development') console.log('endLockTimer: ', isBackground.value, new Date())
//   if (!isBackground.value) {
//     clearTimeout(lockTimer)
//   }
// }
// 定义处理 visibilitychange 事件的函数
// const handleVisibilityChange = () => {
//   if (document.visibilityState === 'hidden') {
//     // 页面进入后台
//     isBackground.value = true
//     startLockTimer()
//   } else {
//     // 页面回到前台
//     isBackground.value = false
//     endLockTimer()
//   }
// };

onBeforeMount(() => {
  changeColor()
  updateTheme()
})

// 在组件挂载时添加事件监听器
// onMounted(() => {
//   document.addEventListener('visibilitychange', handleVisibilityChange);
// });

// // 在组件卸载前移除事件监听器
// onBeforeUnmount(() => {
//   document.removeEventListener('visibilitychange', handleVisibilityChange);
// });
</script>

<style lang="scss">
/* 全站分页统一右对齐（2026-09-14） */
.el-pagination {
  justify-content: flex-end !important;
}
</style>
