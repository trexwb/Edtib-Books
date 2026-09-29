<!--
 * @Author: ${git_name}
 * @Date: 2025-04-18 18:05:33
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-08-01 14:23:11
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsSizeViewer.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="state.loading" class="size-img-container">
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
        v-if="svgDataUrl && !state.error"
        ref="imageViewerRef"
        alt="尺寸图"
        class="size-img"
        :src="svgDataUrl"
        :style="{
          transform: `translate(${position.x}px, ${position.y}px) scale(${scale})`,
          transition: isDragging ? 'none' : 'transform 0.1s ease',
        }"
        title="长按鼠标开始拖拽图片，滚动滑轮放大缩小"
        @error="handleImageError"
        @load="handleImageLoad"
        @mousedown="startDrag"
        @wheel="handleWheel"
      >
        <template #placeholder>
          <div class="size-img-tip">加载中...</div>
        </template>
      </el-image>
      <div v-else-if="state.error && !state.loading" class="size-img-error">
        <el-icon size="28"><icon-picture /></el-icon>
        <span>加载失败</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ZoomOut, RefreshLeft, ZoomIn, Picture as IconPicture } from '@element-plus/icons-vue'
import { evaluateConditionDefault, formatNumber } from '/@/utils/evaluateCondition'

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
const svgUrl = ref(props.productRow?.extension?.svgs || props.productRow?.svgs[0]?.url || '') // SVG 文件的 URL
const svgDataUrl = ref('') // 存储处理后的 SVG 内容

const state = reactive({
  loading: false,
  error: false,
  variables: { ...props.queryForm?.variables },
  unit: props.productRow?.extension?.unit || 'mm',
})
// 图片加载
const handleImageLoad = () => {
  state.loading = false
}
// 图片错误
const handleImageError = () => {
  state.loading = false
  state.error = true
}
// 尺寸图图片缩放比例
const scale = ref(3)
// 拖拽状态
const isDragging = ref(false)
// 图片偏移量
const position = ref({ x: 0, y: +100 })
const dragStart = ref({ x: 0, y: 0 })
// 图片元素的引用
const imageViewerRef = ref(null)
// 开始拖动
const startDrag = (e: any) => {
  e.preventDefault()
  isDragging.value = true

  // 记录鼠标起始位置
  dragStart.value = { x: e.clientX, y: e.clientY }

  // 监听鼠标移动和释放事件
  window.addEventListener('mousemove', onDrag)
  window.addEventListener('mouseup', stopDrag)
}

// 拖动中
const onDrag = (e: any) => {
  if (!isDragging.value) return

  // 计算鼠标移动的距离
  const deltaX = e.clientX - dragStart.value.x
  const deltaY = e.clientY - dragStart.value.y

  // 更新图片位置
  position.value = {
    x: position.value.x + deltaX,
    y: position.value.y + deltaY,
  }

  // 更新拖动起始点
  dragStart.value = { x: e.clientX, y: e.clientY }
}

// 停止拖动
const stopDrag = () => {
  isDragging.value = false

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
const handleWheel = (event: any) => {
  if (!isDragging.value) return
  event.preventDefault()
  if (event.deltaY < 0) {
    if (scale.value < 3) {
      scale.value += 0.1
    }
  } else {
    if (scale.value > 0.5) {
      scale.value -= 0.1
    }
  }
}
/**
 * 放大操作
 * 每次增加0.3倍缩放比例
 * 当前有最大倍数限制注释（暂未启用）
 */
const zoomIn = () => {
  scale.value += 0.3
}
/**
 * 缩小操作
 * 每次减少0.3倍缩放比例
 * 当缩放比例<=0.5倍时显示错误提示
 */
const zoomOut = () => {
  if (scale.value > 0.5) {
    scale.value -= 0.3
  } else {
    $baseMessage('已缩小到最小倍数', 'warning', 'hey')
  }
}
/**
 * 重置视图状态
 * 恢复默认缩放比例（2倍）和位置（0,0）
 */
const resetScale = () => {
  scale.value = 2
  position.value.x = 0
  position.value.y = 0
}
/**
 * 加载并处理SVG文件
 * @param {string} svgPath - SVG文件路径
 * @returns {Promise<void>}
 * 处理流程：
 * 1. 获取SVG原始内容
 * 2. 根据parameters进行占位符替换
 *    - 简单值替换（字符串/数字）
 *    - 条件值替换（ConditionalValue类型）
 *    - 范围条件值替换（对象类型）
 * 3. 生成base64格式的data URL
 */
const loadAndProcessSvg = async (svgPath: string, variables: any): Promise<void> => {
  if (!svgPath) {
    state.error = true
    return
  }
  const { queryForm } = props
  state.loading = true
  state.error = false
  try {
    // 1. 从 URL 获取 SVG 文件内容
    const response = await fetch(svgPath)
    if (!response.ok) throw new Error(`Failed to load SVG: ${response.statusText}`)
    let svgContent = await response.text()
    // 2. 删除通用边框
    // const pattern1 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="1271\\.57,1186\\.27 28428\\.4,1186\\.27\s*" \\/><\\/g>',
    //   'g'
    // );
    // const pattern2 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="28428\\.4,1186\\.27 28428\\.4,19813\\.7\s*" \\/><\\/g>',
    //   'g'
    // );
    // const pattern3 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="28428\\.4,19813\\.7 1271\\.57,19813\\.7\s*" \\/><\\/g>',
    //   'g'
    // );
    // const pattern4 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="1271\\.57,19813\\.7 1271\\.57,1186\\.27\s*" \\/><\\/g>',
    //   'g'
    // );
    // const pattern5 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="29408\\.8,10500 28428\\.4,10500\s*" \\/><\\/g>',
    //   'g'
    // );
    // const pattern6 = new RegExp(
    //   '<g clip-path="url$#clipId0$" fill="none" stroke="rgb$0,0,0$" stroke-width="\\d+"\s*><polyline points="28428\\.4,15057\\.8 1271\\.57,15057\\.8\s*" \\/><\\/g>',
    //   'g'
    // );
    // svgContent = svgContent.replace(/>\s+</g, '><').replace(pattern1, '').replace(pattern2, '').replace(pattern3, '').replace(pattern4, '').replace(pattern5, '').replace(pattern6, '');
    // // 4. 删除边框
    // svgContent = svgContent.replace(/>\s+</g, '><')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="10" />`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="291.176,205.882 29408.8,205.882 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="50" ><polyline points="1271.57,1186.27 28428.4,1186.27 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="30" ><polyline points="1271.57,1186.27 28428.4,1186.27 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="29408.8,205.882 29408.8,20794.1 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="50" ><polyline points="28428.4,1186.27 28428.4,19813.7 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="30" ><polyline points="28428.4,1186.27 28428.4,19813.7 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="29408.8,20794.1 291.176,20794.1 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="50" ><polyline points="28428.4,19813.7 1271.57,19813.7 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="30" ><polyline points="28428.4,19813.7 1271.57,19813.7 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="291.176,20794.1 291.176,205.882 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="50" ><polyline points="1271.57,19813.7 1271.57,1186.27 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="30" ><polyline points="1271.57,19813.7 1271.57,1186.27 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="291.176,10500 1271.57,10500 " /><polyline points="14850,205.882 14850,1186.27 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="30" ><polyline points="29408.8,10500 28428.4,10500 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="25" ><polyline points="29408.8,10500 28428.4,10500 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="20" ><polyline points="28428.4,15057.8 1271.57,15057.8 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="25" ><polyline points="28428.4,15057.8 1271.57,15057.8 " /></g>`, '')
    //   .replace(`<g clip-path="url(#clipId0)" fill="none" stroke="rgb(0,0,0)" stroke-width="13" ><polyline points="14850,19813.7 14850,20794.1 " /><polyline points="683.333,598.039 683.333,2166.67 " /><polyline points="683.333,598.039 2251.96,598.039 " /><polyline points="29016.7,598.039 29016.7,2166.67 " /><polyline points="29016.7,598.039 27448,598.039 " /><polyline points="683.333,20402 683.333,18833.3 " /><polyline points="683.333,20402 2251.96,20402 " /><polyline points="29016.7,20402 29016.7,18833.3 " /><polyline points="29016.7,20402 27448,20402 " /></g>`, '')
    svgContent = svgContent
      .replace(/>\s+</g, '><')
      .replace(`<polyline points="291.176,205.882 29408.8,205.882 " />`, '')
      .replace(`<polyline points="1271.57,1186.27 28428.4,1186.27 " />`, '')
      .replace(`<polyline points="29408.8,205.882 29408.8,20794.1 " />`, '')
      .replace(`<polyline points="28428.4,1186.27 28428.4,19813.7 " />`, '')
      .replace(`<polyline points="29408.8,20794.1 291.176,20794.1 " />`, '')
      .replace(`<polyline points="28428.4,19813.7 1271.57,19813.7 " />`, '')
      .replace(`<polyline points="291.176,20794.1 291.176,205.882 " />`, '')
      .replace(`<polyline points="1271.57,19813.7 1271.57,1186.27 " />`, '')
      .replace(`<polyline points="291.176,10500 1271.57,10500 " />`, '')
      .replace(`<polyline points="14850,205.882 14850,1186.27 " />`, '')
      .replace(`<polyline points="29408.8,10500 28428.4,10500 " />`, '')
      .replace(`<polyline points="28428.4,15057.8 1271.57,15057.8 " />`, '')
      .replace(`<polyline points="14850,19813.7 14850,20794.1 " />`, '')
      .replace(`<polyline points="683.333,598.039 683.333,2166.67 " />`, '')
      .replace(`<polyline points="683.333,598.039 2251.96,598.039 " />`, '')
      .replace(`<polyline points="29016.7,598.039 29016.7,2166.67 " />`, '')
      .replace(`<polyline points="29016.7,598.039 27448,598.039 " />`, '')
      .replace(`<polyline points="683.333,20402 683.333,18833.3 " />`, '')
      .replace(`<polyline points="683.333,20402 2251.96,20402 " />`, '')
      .replace(`<polyline points="29016.7,20402 29016.7,18833.3 " />`, '')
      .replace(`<polyline points="29016.7,20402 27448,20402 " />`, '')
    // 3. 替换变量
    // const variables = queryForm?.variables || {};
    if (variables['Pitch'] || variables['pitch']) {
      svgContent = svgContent.replace('{P}', variables['Pitch'] || variables['pitch'])
    }
    // 当参数中有L时表示长度是指定的，不让选择
    if (!variables['L'] && props.productRow.parameters?.[queryForm.mon]?.['L']) {
      variables['L'] = props.productRow.parameters?.[queryForm.mon]?.['L']
    }
    svgContent = Object.keys(variables).reduce((acc, key) => {
      const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const value = escapedKey == 'd' ? queryForm.mon : variables[key]
      return acc.replace(new RegExp(`{${escapedKey}}`, 'g'), String(value))
    }, svgContent)
    // 4. 将处理后的内容存储到 svgDataUrl
    // console.log('svgContent:', svgContent)
    // svgDataUrl.value = `data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(svgContent)))}`
    // 使用 TextEncoder 处理非 ASCII 字符
    // 5. 将处理后的内容转换为 Data URL (使用 TextDecoder)
    const encoder = new TextEncoder()
    const encoded = encoder.encode(svgContent)
    // const base64 = btoa(String.fromCharCode.apply(null, encoded as unknown as number[]));
    // --- 关键修改：使用 TextDecoder 将 Uint8Array 转换为字符串 ---
    // TextDecoder 专门用于将字节流解码为字符串
    // const textDecoderForBase64 = new TextDecoder('utf-8'); // 或 'latin1'/'iso-8859-1'，对于 base64 编码的原始字节，'latin1' 更精确
    // // 注意：对于 base64 编码，我们通常希望字节被原样解释，'latin1' 映射 0-255 到 U+0000-U+00FF，最符合需求
    // // const textDecoderForBase64 = new TextDecoder('latin1');
    // const byteString = textDecoderForBase64.decode(encoded);
    // 正确方式：将 Uint8Array 转为 byte string（每个字符的 charCode 是 0-255）
    let byteString = ''
    for (let i = 0; i < encoded.length; i++) {
      byteString += String.fromCharCode(encoded[i])
    }
    // 现在 byteString 是一个字符串，每个字符的 charCode 对应一个原始字节
    const base64 = btoa(byteString) // btoa 可以正常处理这个字符串
    svgDataUrl.value = `data:image/svg+xml;base64,${base64}`
  } catch (error) {
    console.error('Error loading or processing SVG:', error)
    state.error = true
  } finally {
    state.loading = false
  }
}

/**
 * 直径变化处理
 * 根据条件切换显示不同的SVG文件：
 * - 当满足drawingLimit条件时显示第二个SVG
 * - 否则显示第一个SVG
 */
const diameterChange = () => {
  const { productRow, queryForm } = props
  if (productRow?.svgs?.length > 1 && productRow?.drawingLimit?.[queryForm?.mon]?.['IMG']) {
    svgUrl.value = evaluateConditionDefault(productRow.drawingLimit[queryForm.mon]['IMG'], productRow.parameters, queryForm)
      ? productRow.svgs[0]?.url || ''
      : productRow.svgs[1]?.url || ''
  } else {
    svgUrl.value = productRow?.extension?.svgs || productRow?.svgs[0]?.url || ''
  }
}
/**
 * 显示SVG入口方法
 * 组合执行直径判断和SVG加载流程
 */
const showImage = () => {
  diameterChange()
  const variables = props.queryForm?.variables || {}
  loadAndProcessSvg(svgUrl.value, variables)
}

const updateUnit = (unit: string) => {
  // state.unit = unit || 'mm';
  const lengthFactor = 25.4
  let variables = {} as any
  handleImageLoad()
  if ((props.productRow?.extension?.unit || 'mm') == unit) {
    variables = { ...props.queryForm?.variables }
  } else {
    if (unit == 'inch') {
      Object.keys(props.queryForm?.variables).map((key: string) => {
        if (key !== 'd' && key !== 'P' && key !== 'Pitch' && !Number.isNaN(Number(props.queryForm?.variables[key]))) {
          variables[key] = formatNumber(props.queryForm?.variables[key] / lengthFactor)
        } else {
          variables[key] = props.queryForm?.variables[key]
        }
      })
    } else if (unit == 'mm') {
      Object.keys(props.queryForm?.variables).map((key: string) => {
        if (key !== 'd' && key !== 'P' && key !== 'Pitch' && !Number.isNaN(Number(props.queryForm?.variables[key]))) {
          variables[key] = formatNumber(props.queryForm?.variables[key] * lengthFactor)
        } else {
          variables[key] = props.queryForm?.variables[key]
        }
      })
    }
  }
  loadAndProcessSvg(svgUrl.value, variables)
}

// 暴露组件方法给父组件
defineExpose({ showImage, updateUnit })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style lang="scss">
.size-img-container {
  width: 100%;
  height: 100%;
  overflow: hidden;

  .size-img-error {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    justify-content: center;
    width: 100%;
    min-height: 200px;
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }

  .size-img-tip {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 50px 0;
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }

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
