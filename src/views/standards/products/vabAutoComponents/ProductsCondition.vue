<!--
 * @Author: ${git_name}
 * @Date: 2025-04-09 16:01:56
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-04-28 18:02:53
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsCondition.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="card-condition-container">
    <div class="card-header">
      <span>已选条件</span>
      <template v-if="standardResult || shapeResult?.length > 0 || categorieResult">:</template>
    </div>
    <el-row :gutter="20">
      <el-col :lg="24" :md="24" :sm="24" style="display: flex; flex-wrap: wrap; gap: 6px; margin-left: -30px" :xl="24" :xs="24">
        <template v-if="categorieResult">
          <el-tag closable size="small" @close="handleCloseCategories">产品分类: {{ categorieResult.names[state.defaultLang] }}</el-tag>
        </template>
        <template v-if="standardResult">
          <el-tag closable size="small" @close="handleCloseStandard">标准分类: {{ standardResult.names[state.defaultLang] }}</el-tag>
        </template>
        <template v-if="shapeResult?.length > 0">
          <el-tag
            v-for="(item, index) in shapeResult"
            :key="`shapeResult[${index}]`"
            closable
            size="small"
            @close="handleCloseShape(item.key, index)"
          >
            {{ item.label + '—' + item.names[state.defaultLang] }}
          </el-tag>
        </template>
      </el-col>
      <el-col :lg="1" :md="1" :sm="1" :xl="1" :xs="1">
        <el-icon
          v-if="standardResult || shapeResult?.length > 0 || categorieResult"
          color="#052965"
          size="18"
          style="cursor: pointer"
          title="清空"
          @click="deleteAll"
        >
          <delete />
        </el-icon>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { Delete } from '@element-plus/icons-vue'

/* 组件模板名称 */
const templateName = 'StandardsProductsCondition'

/* 定义组件选项 */
defineOptions({
  name: templateName, // 注册组件名称，用于调试和keep-alive缓存标识
})

const props = defineProps(['categorieResult', 'standardResult', 'shapeResult'])

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
})

const emit = defineEmits(['handle-change', 'handle-clean']) // 定义组件事件
// // 产品分类已选数据
// const categorieResult = ref<{ id: number | string; parentIds: number[]; title: string } | null>(null)
// // 标准分类已选数据
// const standardResult = reactive<Record<string, any>>({})
// // 形状已选数据
// const shapeResult = ref<{ id: number; title: string; key: string }[]>([])

const handleCloseCategories = () => {
  emit('handle-change', {
    categories: null,
    standards: props.standardResult,
    shape: props.shapeResult,
  })
}

// 标准分类已选条件删除
const handleCloseStandard = () => {
  emit('handle-change', {
    categories: props.categorieResult,
    standards: null,
    shape: props.shapeResult,
  })
}

// 形状已选条件删选
// index 类型放宽：模板 v-for 下标在不同迭代源下可能为 string（内部已做 Number 转换）
const handleCloseShape = (key: any, index: number | string) => {
  props.shapeResult.splice(Number(index), 1)
  nextTick(() => {
    emit('handle-change', {
      categories: props.categorieResult,
      standards: props.standardResult,
      shape: props.shapeResult,
    })
  })
}

// 清空已选条件
const deleteAll = () => {
  emit('handle-clean')
}

const init = () => {}

/* 生命周期钩子 */
onMounted(() => {
  init()
})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {
  init()
})
</script>

<style lang="scss">
.card-condition-container {
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
}
.card-header {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  width: 100px;
  min-height: 22px;

  span {
    font-size: 14px;
    font-weight: bold;
    white-space: nowrap;
  }
}
</style>
