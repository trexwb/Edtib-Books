<!--
 * @Author: trexwb
 * @Date: 2023-11-14 11:28:18
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-08 15:10:30
 * @FilePath: /books/web/library/components/VabAvatar/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2024 by 杭州大美, All Rights Reserved. 
-->
<template>
  <el-dropdown @command="handleCommand" @visible-change="handleVisibleChange">
    <span class="avatar-dropdown">
      <el-avatar class="user-avatar" :src="`${avatar}${avatar.startsWith('file:') ? '' : '!a200'}`">
        <img src="/@/assets/avatar.svg" alt="Default Avatar" />
      </el-avatar>
      <div class="username">
        <span class="hidden-xs-only">{{ username }}</span>
        <vab-icon class="vab-dropdown" :class="{ 'vab-dropdown-active': active }" icon="arrow-down-s-line" />
      </div>
    </span>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="personalCenter">
          <vab-icon icon="account-circle-2-line" />
          <span>{{ translate('个人中心') }}</span>
        </el-dropdown-item>
        <el-dropdown-item command="logout">
          <vab-icon icon="logout-circle-r-line" />
          <span>{{ translate('退出登录') }}</span>
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script lang="ts" setup>
import { ElMessageBox } from 'element-plus'
import { translate } from '/@/i18n'
import { useUserStore } from '/@/store/modules/user'
import { toLoginRoute } from '/@/utils/routes'

defineOptions({
  name: 'VabAvatar',
})

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()
const { avatar, username } = storeToRefs(userStore)
const { logout } = userStore
const active = ref<boolean>(false)

const handleVisibleChange = (value: boolean) => {
  active.value = value
}
const handleCommand = async (command: any) => {
  switch (command) {
    case 'personalCenter':
      await router.push({ path: '/settings/personalCenter' })
      break
    case 'logout':
      ElMessageBox.confirm('您确定要放弃登录退出系统吗？', {
        draggable: false,
        type: 'warning',
        lockScroll: false,
      })
        .then(async () => {
          await logout()
          await router.push(toLoginRoute(route.fullPath))
        })
        .catch(() => {
          // catch error
        })
      break
  }
}
</script>

<style lang="scss" scoped>
.avatar-dropdown {
  display: flex;
  align-content: center;
  align-items: center;
  justify-content: center;
  justify-items: center;

  .user-avatar {
    box-sizing: border-box;
    width: 40px;
    height: 40px;
    margin-left: 15px;
    cursor: pointer;
    border-radius: 50%;
  }

  .username {
    position: relative;
    display: flex;
    align-content: center;
    align-items: center;
    width: max-content;
    height: 40px;
    margin-left: 6px;
    line-height: 40px;
    cursor: pointer;

    [class*='ri-'] {
      margin-left: 0 !important;
    }
  }
}
</style>
