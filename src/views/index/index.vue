<!--
 * @Author: trexwb
 * @Date: 2023-11-14 11:28:18
 * @LastEditors: trexwb
 * @LastEditTime: 2024-11-11 13:58:05
 * @FilePath: /common/console/src/views/index/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2024 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="application-container">
    <el-row :gutter="20">
      <el-col :lg="24" :md="24" :sm="24" :xl="24" :xs="24">
        <recommendation />
      </el-col>
      <el-col :lg="12" :md="12" :sm="24" :xl="12" :xs="24">
        <keep-alive>
          <develop />
        </keep-alive>
      </el-col>
      <el-col :lg="12" :md="12" :sm="24" :xl="12" :xs="24">
        <keep-alive>
          <authorization />
        </keep-alive>
      </el-col>
    </el-row>

    <vab-card skeleton>
      <template #header>
        <vab-icon icon="apps-line" />
        客户端
      </template>
      <template #default>
        点击安装前需按下 Ctrl + F5
        强制刷新当前页面，如果无法安装，PC端请点击浏览器地址栏右侧安装按钮进行安装，手机端请点击添加到主屏幕进行安装，仅支持Edge、Chrome、Safari。Safari如果失败或无法安装可以点击右上角的分享图标，然后选择“添加到主屏幕”，在桌面就能快速打开应用
      </template>
      <template #footer>
        <el-button :disabled="!useInstall" type="primary" @click="handleInstall">点击安装</el-button>
      </template>
    </vab-card>

    <el-divider v-if="currentEnv === 'development'" content-position="left">测试环境出现</el-divider>
    <vab-card v-if="currentEnv === 'development'" skeleton>
      <template #header>用户权限</template>
      <template #default>
        <el-descriptions border :column="2" direction="vertical" width="500px">
          <el-descriptions-item>
            <template #label>账号</template>
            <el-tag>{{ username }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item>
            <template #label>token</template>
            <el-tag>{{ token }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item>
            <template #label>角色</template>
            <vab-json-viewer copyable :expand-depth="5" :value="role" />
          </el-descriptions-item>
          <el-descriptions-item>
            <template #label>权限点</template>
            <vab-json-viewer copyable :expand-depth="5" :value="permission" />
          </el-descriptions-item>
        </el-descriptions>
        <view id="tplapp" />
      </template>
    </vab-card>
  </div>
</template>

<script lang="ts" setup>
import { createApp } from 'vue'
import VabJsonViewer from 'vue-json-viewer'
import { useAclStore } from '/@/store/modules/acl'
import { useUserStore } from '/@/store/modules/user'

// import { getUUID } from '/@/utils/index'
// console.log('uuid:', getUUID())

import ElementPlus from 'element-plus'

defineOptions({
  name: 'Application',
})

const aclStore = useAclStore()
const { role, permission } = storeToRefs(aclStore)
const userStore = useUserStore()
const { username, token } = storeToRefs(userStore)

const currentEnv = process.env.NODE_ENV || 'development'

const useInstall = ref<any>(false)
let deferredPrompt: any

const beforeinstallprompt = () => {
  window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault()
    deferredPrompt = e
    useInstall.value = true
  })
}

const dynamicTemplate = () => {
  /**
   * 假装动态获得的数据
   */
  const template = ref<string>()
  template.value = `<el-descriptions v-if="currentEnv === 'development'" border :column="1" direction="vertical" width="500px">
  <el-descriptions-item>
    <template #label>动态数据</template>
    <el-tag>{{ dynamic }}</el-tag>
  </el-descriptions-item>
</el-descriptions>`
  nextTick(() => {
    createApp({
      data() {
        return {
          currentEnv: currentEnv,
          dynamic: Math.floor(Math.random() * 100),
        }
      },
      template: template.value,
    })
      .use(ElementPlus)
      .mount('#tplapp')
  })
  /**
   * 动态获得模版写入页面
   */
}

onBeforeMount(() => {
  beforeinstallprompt()
  currentEnv === 'development' &&
    nextTick(() => {
      dynamicTemplate()
    })
})

const handleInstall = () => {
  if (deferredPrompt) {
    deferredPrompt.prompt()
    deferredPrompt.userChoice.then(() => {
      deferredPrompt = null
      beforeinstallprompt()
    })
  }
}
</script>

<style lang="scss" scoped>
.index-container {
  :deep() {
    .el-card {
      .el-card__header {
        position: relative;

        .card-header-tag {
          position: absolute;
          top: 15px;
          right: var(--el-margin);
        }

        > div > span {
          display: flex;
          align-items: center;

          i {
            margin-right: 3px;
          }
        }
      }

      .el-card__body {
        position: relative;

        .echarts {
          width: 100%;
          height: 127px;
        }

        .card-footer-tag {
          position: absolute;
          right: var(--el-margin);
          bottom: 15px;
        }
      }
    }
  }
}
</style>
