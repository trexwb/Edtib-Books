<!--
 * @Author: ${git_name}
 * @Date: 2025-04-16 11:51:19
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-08 09:51:51
 * @FilePath: /books/web/src/views/systems/clients/vabAutoComponents/ClientsCheckUpdate.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="container">
    <el-text class="mx-1" type="info">{{ appVersion }}</el-text>
    <div v-show="isUpdateAvailable" class="demo-progress">
      <el-progress :percentage="downloadProgress" status="success" striped striped-flow :stroke-width="24" :text-inside="true" />
    </div>
    <el-button v-show="!isUpdateAvailable" v-loading="isCheckLoading" type="primary" @click="checkUpdate">检查更新</el-button>
    <el-text v-show="!isUpdateAvailable && isCheckUpdate && !isCheckLoading" class="mx-1" type="success">
      目前已经是最新版本，不用更新
    </el-text>
  </div>
</template>

<script lang="ts" setup>
// [迁移调整] 版本/更新相关 IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
import { bridge } from '/@/bridge'

const templateName = 'SettingsCache'
defineOptions({
  name: templateName,
})

const appVersion = ref('')
const isCheckUpdate = ref(false)
const isCheckLoading = ref(false)
const isUpdateAvailable = ref(false)
const downloadProgress = ref(0)

const handleUpdateAvailable = () => {
  // console.log('handleUpdateAvailable');
  isUpdateAvailable.value = true
  isCheckLoading.value = false
}
const handleUpdateNotAvailable = () => {
  // console.log('handleUpdateNotAvailable');
  isUpdateAvailable.value = false
  isCheckLoading.value = false
}
const handleDownloadProgress = (event: any, percent: any) => {
  // console.log('handleDownloadProgress', event, percent);
  downloadProgress.value = Number(Number(percent || downloadProgress.value++).toFixed(1))
}
const handleUpdateDownloaded = (event: any, info: any) => {
  // console.log('handleUpdateDownloaded');
  isUpdateAvailable.value = false
  // 自定义重启
  // window.electronAPI.restartApp(); // 假设你已经定义了一个重启方法
}
const checkUpdate = async () => {
  isCheckUpdate.value = true
  isCheckLoading.value = true
  // [迁移调整] IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
  await bridge.checkUpdate()
}

onMounted(async () => {
  appVersion.value = await bridge.getAppVersion()
  bridge.onUpdateAvailable(handleUpdateAvailable)
  bridge.onUpdateNotAvailable(handleUpdateNotAvailable)
  bridge.onDownloadProgress(handleDownloadProgress)
  bridge.onUpdateDownloaded(handleUpdateDownloaded)
})
</script>

<style lang="scss" scoped>
.container {
  width: 100%;
}

.is-text {
  height: 12px !important;
  padding: 0 !important;
}

.demo-progress .el-progress--line {
  width: 100%;
  margin-bottom: 15px;
}
</style>
