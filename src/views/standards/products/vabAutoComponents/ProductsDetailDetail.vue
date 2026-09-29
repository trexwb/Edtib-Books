<!--
 * @Author: ${git_name}
 * @Date: 2025-04-28 09:45:03
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-29 12:20:07
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailDetail.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-card class="content-preview">
    <template #header>
      <div class="card-header">
        <div>装配说明</div>
      </div>
    </template>
    <el-container style="margin: 20px">
      <el-row :gutter="10">
        <el-col :lg="18" :md="16" :sm="24" :xl="16" :xs="24">
          <v-md-preview v-if="!state.loading" ref="preview" :text="mdText" />
        </el-col>
        <el-col :lg="6" :md="8" :sm="24" :xl="8" :xs="24">
          <video v-if="!!assembliesUrl" :key="videoKey" ref="videoPlayer" controls controlsList="nodownload" muted playsinline width="100%">
            <source :src="assembliesUrl" type="video/mp4" />
            您的浏览器不支持 video 标签。
          </video>
        </el-col>
      </el-row>
    </el-container>
    <!-- <products-assemblies-viewer ref="assembliesViewerRef" :productRow="productRow"></products-assemblies-viewer> -->
  </vab-card>
</template>

<script setup lang="ts">
import VMdPreview from '@kangc/v-md-editor/lib/preview'
import '@kangc/v-md-editor/lib/style/preview.css'
import githubTheme from '@kangc/v-md-editor/lib/theme/github'
import '@kangc/v-md-editor/lib/theme/style/github.css'
import hljs from 'highlight.js'
import { useAclStore } from '/@/store/modules/acl'
import { reactive, ref, computed } from 'vue'

VMdPreview.use(githubTheme, {
  Hljs: hljs,
})

// 定义 props 类型
const props = defineProps(['productRow'])
// const assembliesViewerRef = ref(null)
const assembliesUrl = ref(props.productRow?.assemblies[0]?.url || '') // RENDER 文件的 URL
const videoKey = ref(props.productRow?.id || 0)

// 定义 state 类型
const aclStore = useAclStore()
const state = reactive({
  loading: false,
  defaultLang: computed(() => aclStore.defaultItem.languages || 'zh-cn'),
})

const preview = ref<InstanceType<typeof VMdPreview>>()
const mdText = ref('')

const detailChange = () => {
  assembliesUrl.value = props.productRow?.assemblies[0]?.url || ''
  videoKey.value = props.productRow?.id || 0
  function replaceIdLinks(input: string) {
    const ID_LINK_REGEX = /\[(.*?)\]\(id=(\d+)\)/g
    return input.replace(ID_LINK_REGEX, `[$1](${process.env.NODE_ENV == 'development' ? '/front/' : '/'}standards/products?id=$2)`)
  }
  const detail = props.productRow?.detail ? props.productRow?.detail[state.defaultLang] : ''
  mdText.value = replaceIdLinks(detail)
}

/**
 * 显示渲染图入口方法
 * 组合执行直径判断和渲染图加载流程
 */
const handelDefault = () => {
  state.loading = true
  detailChange()
  state.loading = false
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style lang="scss">
.content-preview {
  .card-header {
    font-size: 14px;
    font-weight: bold;
    color: var(--default-color);
  }

  .github-markdown-body {
    padding: 0 !important;
  }

  .github-markdown-body ol,
  .github-markdown-body ul {
    padding-left: 0 !important;
  }

  .github-markdown-body li > p {
    padding-top: 0 !important;
  }

  .content {
    a {
      font-weight: bold;
      color: var(--default-color);
      text-decoration: underline;
    }

    a[target='_blank'] {
      font-weight: normal;
      color: #000;
      text-decoration: none;
    }
  }
}
</style>
