<!--
 * @Author: ${git_name}
 * @Date: 2025-04-21 15:24:09
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-16 17:27:19
 * @FilePath: /books/web/src/views/settings/PersonalCenter.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="separate-layout-container">
    <div class="hidden-xs-only">
      <el-page-header content="个人设置" title="返回上一页" @back="goBack" />
      <div style="width: 100%; height: 10px"></div>
      <div class="form-box">
        <vab-card v-show="!state.isModify" shadow="always">
          <el-descriptions border class="margin-top" :column="2" size="large" title="我的">
            <el-descriptions-item align="center" label="头像" :rowspan="5" :width="140">
              <el-avatar :size="200" :src="`${editForm.avatar}!a200`">
                <img alt="Default Avatar" src="/@/assets/avatar.svg" />
              </el-avatar>
            </el-descriptions-item>
            <el-descriptions-item>
              <template #label>
                <div class="cell-item">
                  <vab-icon icon="user-follow-line" />
                  姓名
                </div>
              </template>
              {{ editForm.truename }}
              <el-popconfirm class="box-item" placement="right" title="修改个人信息" @confirm="state.isModify = true">
                <template #reference>
                  <vab-icon icon="edit-box-line" />
                </template>
              </el-popconfirm>
            </el-descriptions-item>
            <el-descriptions-item>
              <template #label>
                <div class="cell-item">
                  <vab-icon icon="folder-user-line" />
                  昵称
                </div>
              </template>
              {{ editForm.nickname }}
            </el-descriptions-item>
            <el-descriptions-item>
              <template #label>
                <div class="cell-item">
                  <vab-icon icon="no-credit-card-line" />
                  积分
                </div>
              </template>
              {{ credit }}
            </el-descriptions-item>
            <el-descriptions-item>
              <template #label>
                <div class="cell-item">
                  <vab-icon icon="mail-line" />
                  邮箱
                </div>
              </template>
              {{ editForm.email }}
            </el-descriptions-item>
            <el-descriptions-item>
              <template #label>
                <div class="cell-item">
                  <vab-icon icon="phone-fill" />
                  电话
                </div>
              </template>
              {{ editForm.mobile }}
            </el-descriptions-item>
          </el-descriptions>
        </vab-card>
        <vab-card v-show="state.isModify" shadow="always">
          <template #header>
            <div class="card-header">
              <span>
                <h2>修改个人信息</h2>
              </span>
              <span class="close-btn"><vab-icon icon="arrow-go-back-fill" @click="state.isModify = false" /></span>
            </div>
          </template>
          <el-form label-position="top" :model="editForm" @submit.prevent>
            <el-form-item label="姓名">
              <el-input v-model="editForm.truename" clearable />
            </el-form-item>
            <el-form-item label="昵称">
              <el-input v-model="editForm.nickname" clearable />
            </el-form-item>
            <el-form-item label="邮箱">
              <el-input v-model="editForm.email" clearable />
            </el-form-item>
            <el-form-item label="电话">
              <el-input v-model="editForm.mobile" clearable />
            </el-form-item>
            <el-form-item label="密码">
              <el-input
                v-model.trim="editForm.password"
                clearable
                :placeholder="translate('请输入新密码以修改密码，留空则表示不修改密码！')"
                show-password
                type="password"
              />
            </el-form-item>
            <el-form-item label="图像" style="width: 100%">
              <products-upload accept="image/*" :files="form.avatar" :limit="1" name="avatar" @change-files="changeFiles" />
            </el-form-item>
            <div style="margin-top: 20px; text-align: center">
              <el-button native-type="submit" type="primary" @click="onUserSave">保存</el-button>
            </div>
          </el-form>
        </vab-card>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { translate } from '/@/i18n'
import { layout } from '/@/config/'
import { usersSave } from '/@/api/authorize'
import { useSettingsStore } from '/@/store/modules/settings'
import { useUserStore } from '/@/store/modules/user'

const userStore = useUserStore()
const { setAvatar } = useUserStore()
const { avatar, username, email, mobile, truename, credit } = storeToRefs(userStore)

defineOptions({
  name: 'PersonalCenter',
})

const $baseMessage = inject<any>('$baseMessage') // 注入全局消息提示方法
const settingsStore = useSettingsStore()
const route = useRoute()
const { device, theme } = storeToRefs(settingsStore)

const goBack = async () => {
  await history.back()
}

const state = reactive<any>({
  loading: false, // 数据加载状态
  isModify: false,
})

const editForm = reactive<any>({
  truename: truename.value,
  nickname: username.value,
  email: email.value,
  mobile: mobile.value,
  password: '',
  avatar: avatar.value,
})
const form = reactive<any>({
  avatar: avatar.value ? [{ url: avatar.value }] : [],
})

const changeFiles = (uploadFiles: any, uploadName?: string) => {
  form[uploadName || 'covers'] = uploadFiles.map((item: any) => ({
    name: item.name || '',
    url: item.url || '',
  }))
}
const onUserSave = async () => {
  editForm.avatar = form.avatar.length > 0 ? form.avatar[0].url : ''
  await usersSave(editForm)
  $baseMessage('保存成功', 'success')
  setAvatar(editForm.avatar)
  goBack()
}

watch(
  route,
  () => {
    if (device.value !== 'mobile') {
      if (route.path === '/settings/personalCenter') {
        theme.value.layout = 'horizontal'
      } else {
        if (localStorage.getItem('shop-vite-theme')) {
          theme.value.layout = JSON.parse(localStorage.getItem('shop-vite-theme') as string).layout || layout
        } else {
          theme.value.layout = layout
        }
      }
    } else {
      theme.value.layout = 'vertical'
    }
  },
  { immediate: true }
)
</script>

<style lang="scss" scoped>
.separate-layout-container {
  h2 {
    padding: 0;
    margin: 0;
  }

  width: 100%;

  .form-box {
    width: 80%;
    margin: 0 auto;
  }

  :deep() {
    .tile-container {
      min-height: calc(var(--el-container-height) - 150px);
      padding: var(--el-padding) !important;
      border-radius: var(--el-border-radius-base);
    }
  }
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
</style>
