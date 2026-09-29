<!--
 * @Author: ${git_name}
 * @Date: 2025-05-09 09:39:14
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-20 18:25:56
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasTensileLoad.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-card>
    <template #header>
      <div class="card-header">
        <div>{{ currentFormula.name }}</div>
        <el-icon @click="closeFormula">
          <close />
        </el-icon>
      </div>
    </template>
    <el-form class="demo-form-inline" :inline="true">
      <el-form-item label="选择">
        <el-select v-model="formulasForm.intensity" placeholder="" style="width: 150px" @change="changeResult">
          <template #prefix>强度等级</template>
          <el-option v-for="item in state.intensity" :key="`intensity[${item.value}]`" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="=" label-width="5px">
        <el-text style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(result || 0) }}
        </el-text>
        <el-icon
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="handleCopy(formatNumber(result || 0), currentFormula.name)"
        >
          <document-copy />
        </el-icon>
      </el-form-item>
      <el-form-item>
        <template #label>
          <div>结果换算:</div>
        </template>
        <el-switch
          v-model="metric"
          active-text="KN"
          active-value="KN"
          inactive-text="lbs"
          inactive-value="lbs"
          inline-prompt
          style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
          width="80"
          @change="changeUnit"
        />
      </el-form-item>
    </el-form>
  </vab-card>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { DocumentCopy, Close } from '@element-plus/icons-vue'
import { evaluateFormula, formatNumber } from '/@/utils/evaluateCondition'
import clip from '/@/utils/clipboard'

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { tolerance, intensity, material } = storeToRefs(aclStore) // 解构多语言列表响应式引用

/* 事件和全局方法 */
const emit = defineEmits(['close']) // 定义组件事件
// 定义 props 类型
const props = defineProps(['currentFormula', 'productRow', 'queryForm', 'formulas'])

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  submit: false,
  intensity: [],
  defaultFormulasForm: {
    intensity: '',
  },
})

const conditionValidate = (A: number[], B: any) => {
  if (!B) return true
  // 将 A 转换为 Set 提高查找效率
  const setA = new Set(A)
  for (const item of B) {
    if (typeof item === 'string' && item.startsWith('!(')) {
      // 处理否定条件：!(x|y|z)
      const content = item.slice(2, -1) // 去掉 "!(" 和 ")"
      const options = content.split('|').map(Number)
      const hasAny = options.some((option) => setA.has(option))
      if (hasAny) return false // 否定条件中只要有一个匹配，整体失败
    } else if (typeof item === 'string' && item.includes('|')) {
      // 处理 "x|y|z" 格式
      const options = item.split('|').map(Number)
      const hasAny = options.some((option) => setA.has(option))
      if (!hasAny) return false
    } else {
      // 普通值必须存在
      const numericValue = typeof item === 'string' ? Number(item) : item
      if (!setA.has(numericValue)) return false
    }
  }
  return true
}

const formulasForm = ref<any>(JSON.parse(JSON.stringify(state.defaultFormulasForm)))
const metric = ref(props.currentFormula?.unit || 'KN')
const result = ref(0)

// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}
const changeUnit = () => {
  const lbsLoadFactor = 0.004536
  if (metric.value == 'lbs') {
    result.value = result.value / lbsLoadFactor
  } else if (metric.value == 'KN') {
    result.value = result.value * lbsLoadFactor
  }
}

// 计算全部公式
const changeResult = () => {
  if (state.submit) return
  state.submit = true
  result.value = 0
  try {
    // formulasForm.exteriors formulasForm.lubricate
    // 缓存高频访问的响应式值
    const currentIntensity = intensity.value
    const intensityKey = formulasForm.value.intensity
    const intensityVal = currentIntensity[intensityKey] || {}

    // 防御性拷贝原始variables
    const originalVariables = props.queryForm?.variables || {}
    const variables = { ...originalVariables }

    // 将强度对应的参数过滤掉密度和条件后作为变量使用
    Object.keys(intensityVal)
      .filter((item: string) => !['material', 'condition'].includes(item))
      .forEach((item: string) => {
        variables[item] = intensityVal[item]
      })

    // 计算固定公式
    result.value = evaluateFormula(props.currentFormula?.columnar, props.formulas, variables)
    if (metric.value !== props.currentFormula?.unit) changeUnit()
  } catch (e) {
    console.log(`固定公式[${props.currentFormula.code}]:`, e, props.currentFormula?.columnar)
  } finally {
    state.submit = false
  }
}

const closeFormula = () => {
  emit('close')
}

const handelDefault = () => {
  result.value = 0
  state.intensity = []
  // options键是用于显示提供选择的强度
  ;(intensity.value['options'] || []).forEach((key: string) => {
    if (
      conditionValidate(props.productRow?.shapes_id, intensity.value[key]?.condition?.shapesIds) &&
      conditionValidate(props.productRow?.categories_id, intensity.value[key]?.condition?.categoriesIds)
    ) {
      state.intensity.push({
        value: key,
        label: `${key}`,
      })
    }
  })
  changeResult()
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })
</script>

<style lang="scss" scoped></style>
