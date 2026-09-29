<!--
 * @Author: trexwb
 * @Date: 2024-11-08 13:40:10
 * @LastEditors: trexwb
 * @LastEditTime: 2024-11-08 17:32:42
 * @FilePath: /common/console/src/views/index/vabAutoComponents/Recommendation.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2024 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-card>
    <template #header>
      <vab-icon icon="reserved-line" />
      快捷菜单
    </template>

    <el-row :gutter="20">
      <el-col v-for="(item, index) in iconList" :key="index" :lg="6" :md="8" :sm="8" :xl="6" :xs="24">
        <vab-link :to="item.link">
          <vab-card class="icon-panel">
            <el-badge class="item" :value="item.value">
              <vab-icon :icon="item.icon" />
            </el-badge>
            <div class="icon-panel-title">
              {{ item.title }}
              <div class="icon-panel-tips">{{ item.tips }}</div>
            </div>
          </vab-card>
        </vab-link>
      </el-col>
    </el-row>
  </vab-card>

</template>

<script lang="ts" setup>
import { useAclStore } from '/@/store/modules/acl'

const aclStore = useAclStore()
const { role } = storeToRefs(aclStore)

interface RoleItem {
  icon: string;
  title: string;
  tips: string;
  link: string;
  value: string;
}

const roleList: any = {
  attachmentConfigs: {
    icon: 'file-settings-line',
    title: '上传配置',
    tips: '上传文件的相关配置',
    link: '/cms/attachment/configs',
    value: '',
  },
  attachmentFiles: {
    icon: 'file-image-line',
    title: '附件管理',
    tips: '上传文件的查询管理',
    link: '/cms/attachment/files',
    value: '',
  },
  simpleCMSLanguages: {
    icon: 'translate',
    title: '多语言',
    tips: '系统设置多语言能力',
    link: '/cms/simpleCMS/languages',
    value: '',
  },
  simpleCMSCustoms: {
    icon: 'plug-line',
    title: '自定义',
    tips: '可以自己定义相关字段',
    link: '/cms/simpleCMS/customs',
    value: '',
  },
  simpleCMSColumns: {
    icon: 'folder-3-line',
    title: '栏目',
    tips: '网站栏目的管理',
    link: '/cms/simpleCMS/columns',
    value: '',
  },
  simpleCMSCategories: {
    icon: 'menu-2-line',
    title: '分类管理',
    tips: '可以自己定义相关字段',
    link: '/cms/simpleCMS/categories',
    value: '',
  },
  simpleCMSArticles: {
    icon: 'article-line',
    title: '文章管理',
    tips: '可以自己定义相关字段',
    link: '/cms/simpleCMS/articles',
    value: '',
  },
  simpleCMSDocs: {
    icon: 'archive-stack-line',
    title: '常用文档',
    tips: '显示在栏目下的相关内容',
    link: '/cms/simpleCMS/docs',
    value: '',
  }
}

const iconList: RoleItem[] = []
const roles = unref(role)
if (roles) {
  roles.forEach((item: string) => {
    if (roleList[item]) {
      iconList.push(roleList[item])
    }
  });
}
</script>

<style lang="scss" scoped>
.icon-panel {
  margin-bottom: 8px;
  cursor: pointer;
  border: 0 !important;

  :deep() {
    .el-card__body {
      height: 65px;
      padding: 10px;

      &:hover {
        i {
          color: var(--el-color-white);
          background: var(--el-color-primary);
        }
      }

      i {
        display: inline-block;
        width: 50px;
        height: 50px;
        font-size: 30px;
        line-height: 50px;
        color: var(--el-color-primary);
        background: var(--el-color-primary-light-9);
        border-radius: var(--el-border-radius-base);
        transition: all ease-in-out 0.3s;
      }

      .icon-panel-title {
        display: inline-block;
        padding-top: 10px;
        margin-left: 10px;
        vertical-align: -10px;

        .icon-panel-tips {
          margin-top: 5px;
          font-size: var(--el-font-size-small);
          color: var(--el-color-grey);
        }
      }
    }
  }
}
</style>
