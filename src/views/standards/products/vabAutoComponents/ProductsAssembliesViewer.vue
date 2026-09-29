<!--
 * @Author: ${git_name}
 * @Date: 2025-05-16 09:21:39
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-16 09:31:03
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsAssembliesViewer.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="isLoading" class="size-img-container">
    <div class="size-video-box">
      <video ref="videoPlayer" class="video" controls height="100%">
        <source :src="assembliesUrl" type="video/mp4" />
        您的浏览器不支持 video 标签。
      </video>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ZoomOut, RefreshLeft, ZoomIn } from '@element-plus/icons-vue'
import { evaluateConditionDefault } from '/@/utils/evaluateCondition'

const props = defineProps(['productRow', 'queryForm'])
const assembliesUrl = ref(props.productRow?.assemblies[0]?.url || '') // RENDER 文件的 URL

const isLoading = ref(false)
// 图片加载
const handleRenderImageLoad = () => {
  isLoading.value = false
}
// 图片错误
const handleRenderImageError = () => {
  isLoading.value = false
}
/**
 * 显示渲染图入口方法
 * 组合执行直径判断和渲染图加载流程
 */
const showImage = () => {
  handleRenderImageLoad()
  assembliesUrl.value = props.productRow?.assemblies[0]?.url || ''
  handleRenderImageError()
}

// 暴露组件方法给父组件
defineExpose({ showImage })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style scoped lang="scss">
.size-img-container {
  width: 100%;
  height: 100%;
  overflow: hidden;

  .el-image__error {
    padding: 50px 0;
  }

  .operate-box {
    position: relative;
    z-index: 100;
    width: 100%;
    height: 40px;
    padding-top: 10px;
    text-align: center;
    background: #eaf0f0;

    .operate-cion {
      margin-right: 30px;
      font-size: 20px;
      cursor: pointer;
    }
  }

  .size-video-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 10px;
    overflow: hidden;

    .size-img {
      position: absolute;
      z-index: 50;
      width: 500px;
      height: auto;
      cursor: pointer;
      user-select: none;
      transition: all 0.3s ease-in-out;
    }
  }
}
</style>
