<!--
 * @Author: ${git_name}
 * @Date: 2025-04-12 11:21:20
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-04 14:43:40
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailBasic.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <el-card shadow="always" style="width: 100%">
    <div v-show="productRow.names['en']" class="en-name">{{ productRow.names['en'] || '' }}</div>
    <el-tabs v-model="state.topPictureTabsActive">
      <el-tab-pane label="尺寸图" name="Size">
        <template #label>
          <el-text class="tabs-label">尺寸图</el-text>
        </template>
        <template #default>
          <div v-if="!!productRow?.svgs" class="tabs-content">
            <products-size-viewer ref="sizeViewerRef" :product-row="productRow" :query-form="queryForm" />
            <products-right-content
              :extension="productRow.extension"
              :parameters="productRow.parameters"
              :query-form="queryForm"
              :variables="variables"
              @update-unit="updateUnit"
            />
          </div>
        </template>
      </el-tab-pane>
      <el-tab-pane label="三维图" name="3D">
        <template #label>
          <el-text class="tabs-label">三维图</el-text>
        </template>
        <template #default>
          <div v-if="!!productRow?.models" class="tabs-content">
            <products-stl-viewer ref="stlViewerRef" :product-row="productRow" :query-form="queryForm" />
            <products-right-content
              :extension="productRow.extension"
              :parameters="productRow.parameters"
              :query-form="queryForm"
              :variables="variables"
              @update-unit="updateUnit"
            />
          </div>
        </template>
      </el-tab-pane>
      <el-tab-pane label="渲染图" name="Renders">
        <template #label>
          <el-text class="tabs-label">渲染图</el-text>
        </template>
        <template #default>
          <div v-if="!!productRow?.renders" class="tabs-content">
            <products-render-viewer ref="renderViewerRef" :product-row="productRow" :query-form="queryForm" />
            <products-right-content
              :extension="productRow.extension"
              :parameters="productRow.parameters"
              :query-form="queryForm"
              :variables="variables"
              @update-unit="updateUnit"
            />
          </div>
        </template>
      </el-tab-pane>
    </el-tabs>
    <products-size-filter
      ref="sizeFilterRef"
      :diameter-length="productRow.diameterLength"
      :parameters="productRow.parameters"
      :query-form="queryForm"
      :tolerance="productRow.tolerance"
      @handel-change="handelChange"
      @update-length="updateLength"
      @update-query-form="updateQueryForm"
    />
  </el-card>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { getVariables } from '/@/utils/evaluateCondition'

const templateName = 'StandardsProductsDetailBasic'

defineOptions({
  name: templateName,
})

const sizeViewerRef = ref<any>(null)
const stlViewerRef = ref<any>(null)
const renderViewerRef = ref<any>(null)
const sizeFilterRef = ref<any>(null)

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { languages, defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用

const emit = defineEmits(['updateLength', 'updateQueryForm'])
const props = defineProps(['productRow', 'queryForm', 'variables'])

const state = reactive<any>({
  topPictureTabsActive: 'Size',
  activeName: defaultItem.value.languages || 'zh-cn', // 当前激活的标签页
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
})

const updateQueryForm = (data: any) => {
  emit('updateQueryForm', data)
}

const updateLength = (searchLen: number) => {
  emit('updateLength', Number(searchLen))
}

const handelChange = async () => {
  if (props.queryForm.mon) {
    props.queryForm.variables = getVariables(props.productRow.parameters || {}, props.productRow.tolerance || {}, props.queryForm)
  }
  nextTick(() => {
    sizeViewerRef.value?.showImage()
    renderViewerRef.value?.showImage()
    stlViewerRef.value?.showImage()
  })
}

const updateUnit = (unit: string) => {
  sizeViewerRef.value.updateUnit(unit)
}

const handelDefault = async () => {
  sizeFilterRef.value.handelDefault()
}

// 暴露组件方法
defineExpose({ handelDefault })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style lang="scss" scoped>
.tabs-content {
  position: relative;
  width: 100%;
  height: 600px;
  overflow: hidden;
  background: #eaf0f0;
  border: 1px solid var(--default-color);
}
</style>
