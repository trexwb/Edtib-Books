<!--
 * @Author: ${git_name}
 * @Date: 2025-05-08 16:18:19
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-20 18:07:59
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasCommon.vue
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
      <el-form-item :label="`${currentFormula.name} =`">
        <el-text style="font-size: 28px; font-weight: bold" type="primary">
          {{
            Number(result || 0)
              .toFixed(3)
              .toString()
          }}
        </el-text>
        <el-icon
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="
            handleCopy(
              Number(result || 0)
                .toFixed(3)
                .toString(),
              currentFormula.name
            )
          "
        >
          <document-copy />
        </el-icon>
      </el-form-item>
      <el-form-item v-if="!!currentFormula.unit">
        <template #label>
          <div>结果换算:</div>
        </template>
        <el-switch
          v-if="currentFormula.unit == 'N*m' || currentFormula.unit == 'kg*f*cm'"
          v-model="metric"
          active-text="N*m"
          active-value="N*m"
          inactive-text="kg*f*cm"
          inactive-value="kg*f*cm"
          inline-prompt
          style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
          width="80"
          @change="changeUnit"
        />
        <el-switch
          v-if="currentFormula.unit == 'mm' || currentFormula.unit == 'inch'"
          v-model="metric"
          active-text="公制(mm)"
          active-value="mm"
          inactive-text="美制(inch)"
          inactive-value="inch"
          inline-prompt
          style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
          width="80"
          @change="changeUnit"
        />
        <el-switch
          v-if="currentFormula.unit == 'KN' || currentFormula.unit == 'lbs'"
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
import { evaluateConditionDefault, evaluateFormula } from '/@/utils/evaluateCondition'
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
  material: [],
  intensity: [],
  defaultFormulasForm: {
    material: '',
    money: '',
    intensity: '',
  },
})

const metric = ref(props.currentFormula?.unit || 'N*m')
const result = ref<number>(0)

// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}

const changeUnit = () => {
  // console.log(value, code)
  const lengthFactor = 25.4
  const loadFactor = 9.8
  const lbsLoadFactor = 0.004536

  if (metric.value == 'inch') {
    result.value = result.value / lengthFactor
  } else if (metric.value == 'mm') {
    result.value = result.value * lengthFactor
  } else if (metric.value == 'N*m') {
    result.value = result.value / loadFactor
  } else if (metric.value == 'kg*f*cm') {
    result.value = result.value * loadFactor
  } else if (metric.value == 'lbs') {
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
    // 防御性拷贝原始variables
    const originalVariables = props.queryForm?.variables || {}
    const variables = { ...originalVariables }

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
  changeResult()
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })
</script>

<style lang="scss" scoped></style>
