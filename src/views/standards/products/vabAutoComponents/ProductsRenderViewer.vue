<!--
 * @Author: ${git_name}
 * @Date: 2025-04-09 08:45:50
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-04 10:52:58
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsRenderViewer.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="isLoading" class="size-img-container">
    <div class="operate-box">
      <el-icon class="operate-cion" title="放大" @click="zoomIn">
        <zoom-in />
      </el-icon>
      <el-icon class="operate-cion" title="还原" @click="resetScale">
        <refresh-left />
      </el-icon>
      <el-icon class="operate-cion" title="缩小" @click="zoomOut">
        <zoom-out />
      </el-icon>
    </div>
    <div class="size-img-box">
      <el-image
        v-if="!isLoading"
        ref="imageRenderRef"
        alt="尺寸图"
        class="size-img"
        :src="renderUrl"
        :style="{
          transform: `translate(${renderPosition.x}px, ${renderPosition.y}px) scale(${renderScale})`,
          transition: renderDragging ? 'none' : 'transform 0.1s ease',
        }"
        title="长按鼠标开始拖拽图片，滚动滑轮放大缩小"
        @error="handleRenderImageError"
        @load="handleRenderImageLoad"
        @mousedown="startDrag"
        @wheel="handleRenderWheel"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ZoomOut, RefreshLeft, ZoomIn } from '@element-plus/icons-vue'
import { evaluateConditionDefault } from '/@/utils/evaluateCondition'

const $baseMessage = inject<any>('$baseMessage')

/**
 * 条件值类型定义
 * @property {string} condition - 条件表达式字符串
 * @property {string} [A] - 条件为false时的替换值
 * @property {string} [B] - 条件为true时的替换值
 */
interface ConditionalValue {
  condition: string
  A?: string
  B?: string
}

const props = defineProps(['productRow', 'queryForm'])
const renderUrl = ref(props.productRow?.extension?.renders || props.productRow?.renders[0]?.url || '') // RENDER 文件的 URL

const isLoading = ref(false)
// 图片加载
const handleRenderImageLoad = () => {
  isLoading.value = false
}
// 图片错误
const handleRenderImageError = () => {
  isLoading.value = false
}
// 尺寸图图片缩放比例
const renderScale = ref(1)
// 图片偏移量
const renderPosition = ref({ x: 0, y: 0 })
// 拖拽状态
const renderDragging = ref(false)
const dragRenderStart = ref({ x: 0, y: 0 })
// 图片元素的引用
const imageRenderRef = ref(null)

// 开始拖动
const startDrag = (e: any) => {
  e.preventDefault()
  renderDragging.value = true

  // 记录鼠标起始位置
  dragRenderStart.value = { x: e.clientX, y: e.clientY }

  // 监听鼠标移动和释放事件
  window.addEventListener('mousemove', onDrag)
  window.addEventListener('mouseup', stopDrag)
}

// 拖动中
const onDrag = (e: any) => {
  if (!renderDragging.value) return

  // 计算鼠标移动的距离
  const deltaX = e.clientX - dragRenderStart.value.x
  const deltaY = e.clientY - dragRenderStart.value.y

  // 更新图片位置
  renderPosition.value = {
    x: renderPosition.value.x + deltaX,
    y: renderPosition.value.y + deltaY,
  }

  // 更新拖动起始点
  dragRenderStart.value = { x: e.clientX, y: e.clientY }
}

// 停止拖动
const stopDrag = () => {
  renderDragging.value = false

  // 移除鼠标移动和释放事件监听器
  window.removeEventListener('mousemove', onDrag)
  window.removeEventListener('mouseup', stopDrag)
}
/**
 * 鼠标滚轮事件处理
 * @param {WheelEvent} event - 滚轮事件对象
 * 功能说明：
 * - 仅在拖拽状态下生效
 * - 上滚放大（增加0.1倍）
 * - 下滚缩小（减少0.1倍）
 * - 缩放范围限制在0.5-3倍之间
 */
const handleRenderWheel = (event: any) => {
  // if (!renderDragging.value) return
  event.preventDefault()
  if (event.deltaY < 0) {
    if (renderScale.value < 3) {
      renderScale.value += 0.1
    }
  } else {
    if (renderScale.value > 0.5) {
      renderScale.value -= 0.1
    }
  }
}

/**
 * 放大操作
 * 每次增加0.3倍缩放比例
 * 当前有最大倍数限制注释（暂未启用）
 */
const zoomIn = () => {
  renderScale.value += 0.3
}

/**
 * 缩小操作
 * 每次减少0.3倍缩放比例
 * 当缩放比例<=0.5倍时显示错误提示
 */
const zoomOut = () => {
  if (renderScale.value > 0.5) {
    renderScale.value -= 0.3
  } else {
    $baseMessage('已缩小到最小倍数', 'warning', 'hey')
  }
}

/**
 * 重置视图状态
 * 恢复默认缩放比例（2倍）和位置（0,0）
 */
const resetScale = () => {
  renderScale.value = 1
  renderPosition.value.x = 0
  renderPosition.value.y = 0
}

/**
 * 直径变化处理
 * 根据条件切换显示不同的RENDER文件：
 * - 当满足drawingLimit条件时显示第二个RENDER
 * - 否则显示第一个RENDER
 */
const diameterChange = () => {
  const { productRow, queryForm } = props
  if (productRow?.renders.length > 1 && productRow?.drawingLimit?.[queryForm?.mon]?.['IMG']) {
    renderUrl.value = evaluateConditionDefault(productRow.drawingLimit[queryForm.mon]['IMG'], productRow.parameters, queryForm)
      ? productRow.renders[0]?.url || ''
      : productRow.renders[1]?.url || ''
  } else {
    renderUrl.value = productRow?.extension?.renders || productRow?.renders[0]?.url || ''
  }
}
/**
 * 显示渲染图入口方法
 * 组合执行直径判断和渲染图加载流程
 */
const showImage = () => {
  handleRenderImageLoad()
  diameterChange()
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

  .size-img-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: calc(100% - 40px);
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
