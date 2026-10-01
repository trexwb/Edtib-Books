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
    <el-button v-show="updateReady" v-loading="installing" type="success" @click="installUpdate">重启并安装</el-button>
    <el-text v-show="!isUpdateAvailable && isCheckUpdate && !isCheckLoading" class="mx-1" type="success">
      目前已经是最新版本，不用更新
    </el-text>
  </div>
</template>

<script lang="ts" setup>
// [迁移调整] 版本/更新相关 IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
import { ElMessage } from 'element-plus'
import { bridge } from '/@/bridge'

const templateName = 'SettingsCache'
defineOptions({
  name: templateName,
})

const appVersion = ref('')
const isCheckUpdate = ref(false)
const isCheckLoading = ref(false)
const isUpdateAvailable = ref(false)
const updateReady = ref(false)
const installing = ref(false)
const downloadProgress = ref(0)

const disposers: Array<() => void> = []

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
const handleDownloadProgress = (event: any, payload: any) => {
  // Rust 侧 emit 的是 { percent, transferred, total } 对象，直接当数字使用会得到 NaN
  downloadProgress.value = Number((payload?.percent ?? 0).toFixed(1))
}
const handleUpdateDownloaded = () => {
  downloadProgress.value = 100
  updateReady.value = true
}
const checkUpdate = async () => {
  isCheckUpdate.value = true
  isCheckLoading.value = true
  updateReady.value = false
  downloadProgress.value = 0
  // [迁移调整] IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
  await bridge.checkUpdate()
}
const installUpdate = async () => {
  installing.value = true
  try {
    // 安装已下载的更新包（Rust confirm_update → Update::install），完成后应用自动重启
    await bridge.confirmUpdate()
  } catch (error: any) {
    installing.value = false
    ElMessage({ message: error?.message ?? String(error), type: 'error' })
  }
}

onMounted(async () => {
  appVersion.value = await bridge.getAppVersion()
  disposers.push(
    bridge.onUpdateAvailable(handleUpdateAvailable),
    bridge.onUpdateNotAvailable(handleUpdateNotAvailable),
    bridge.onDownloadProgress(handleDownloadProgress),
    bridge.onUpdateDownloaded(handleUpdateDownloaded)
  )
})

onBeforeUnmount(() => {
  // 注销事件监听，避免页面反复进出后同一事件触发多次回调
  disposers.splice(0).forEach((dispose) => dispose())
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
