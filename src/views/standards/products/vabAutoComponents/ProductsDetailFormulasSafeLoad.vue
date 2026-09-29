<!--
 * @Author: ${git_name}
 * @Date: 2025-05-09 11:07:00
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-25 17:18:24
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasSafeLoad.vue
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
          <el-option v-for="item in state.intensity" :key="`intensity[${item}]`" :label="item" :value="item" />
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
  intensity: ['04', '05', '5', '6', '8', '10', '12'],
  defaultFormulasForm: {
    intensity: '',
  },
})

// 公式表单 d->p->J
const safeVaule = ref<any>({
  '5': { '0.8': { '04': 5.4, '05': 7.1, '5': 8.25, '6': 9.5, '8': 12.14, '10': 14.8, '12': 16.3 } },
  '6': { '1': { '04': 7.64, '05': 10, '5': 11.7, '6': 13.5, '8': 17.2, '10': 20.9, '12': 23.1 } },
  '7': { '1': { '04': 11, '05': 14.5, '5': 16.8, '6': 19.4, '8': 24.7, '10': 30.1, '12': 33.2 } },
  '8': {
    '1': { '04': 14.9, '05': 19.6, '5': 27, '6': 30.2, '8': 37.4, '10': 43.1, '12': 47 },
    '1.25': { '04': 13.9, '05': 18.3, '5': 21.6, '6': 24.9, '8': 31.8, '10': 38.1, '12': 42.5 },
  },
  '10': {
    '1': { '04': 24.5, '05': 32.2, '5': 44.5, '6': 49.7, '8': 61.6, '10': 71, '12': 77.4 },
    '1.5': { '04': 22, '05': 29, '5': 34.2, '6': 39.4, '8': 50.5, '10': 60.3, '12': 67.3 },
    '1.25': { '04': 23.3, '05': 30.6, '5': 44.2, '6': 47.1, '8': 58.4, '10': 67.3, '12': 73.4 },
  },
  '12': {
    '1.75': { '04': 32, '05': 42.2, '5': 51.4, '6': 59, '8': 74.2, '10': 88.5, '12': 100.3 },
    '1.5': { '04': 33.5, '05': 44, '5': 60.8, '6': 68.7, '8': 84.1, '10': 97.8, '12': 105.7 },
    '1.25': { '04': 35, '05': 46, '5': 63.5, '6': 71.8, '8': 88, '10': 102.2, '12': 110.5 },
  },
  '14': {
    '2': { '04': 43.7, '05': 57.5, '5': 70.2, '6': 80.5, '8': 101.2, '10': 120.8, '12': 136.9 },
    '1.5': { '04': 47.5, '05': 62.5, '5': 86.3, '6': 97.5, '8': 119.4, '10': 138.8, '12': 150 },
  },
  '16': {
    '2': { '04': 59.7, '05': 78.5, '5': 95.8, '6': 109.9, '8': 138.2, '10': 164.9, '12': 186.8 },
    '1.5': { '04': 63.5, '05': 83.5, '5': 115.2, '6': 130.3, '8': 159.5, '10': 185.4, '12': 200.4 },
  },
  '18': {
    '2': { '04': 77.5, '05': 102, '5': 146.9, '6': 177.5, '8': 210.1, '10': 220.3, '12': 0 },
    '2.5': { '04': 73, '05': 96, '5': 121, '6': 138.2, '8': 176.6, '10': 203.5, '12': 230.4 },
    '1.5': { '04': 81.7, '05': 107.5, '5': 154.8, '6': 187, '8': 221.5, '10': 232.2, '12': 0 },
  },
  '20': {
    '2': { '04': 98, '05': 129, '5': 185.8, '6': 224.5, '8': 265.7, '10': 278.6, '12': 0 },
    '2.5': { '04': 93.1, '05': 122.5, '5': 154.4, '6': 176.4, '8': 225.4, '10': 259.7, '12': 294 },
    '1.5': { '04': 103.4, '05': 136, '5': 195.8, '6': 236.6, '8': 280.2, '10': 293.8, '12': 0 },
  },
  '22': {
    '2': { '04': 120.8, '05': 159, '5': 229, '6': 276.7, '8': 327.5, '10': 343.4, '12': 0 },
    '2.5': { '04': 115.1, '05': 151.5, '5': 190.9, '6': 218.2, '8': 278.8, '10': 321.2, '12': 363.6 },
    '1.5': { '04': 126.5, '05': 166.5, '5': 239.8, '6': 289.7, '8': 343, '10': 359.6, '12': 0 },
  },
  '24': {
    '2': { '04': 145.9, '05': 192, '5': 276.5, '6': 334.1, '8': 395.5, '10': 414.7, '12': 0 },
    '3': { '04': 134.1, '05': 176.5, '5': 222.4, '6': 254.2, '8': 324.8, '10': 374.2, '12': 423.6 },
  },
  '27': {
    '2': { '04': 188.5, '05': 248, '5': 351.1, '6': 431.5, '8': 510.9, '10': 536.7, '12': 0 },
    '3': { '04': 174.4, '05': 229.5, '5': 289.2, '6': 330.5, '8': 422.3, '10': 486.5, '12': 550.8 },
  },
  '30': {
    '2': { '04': 236, '05': 310.5, '5': 447.1, '6': 540.3, '8': 639.6, '10': 670.7, '12': 0 },
    '3.5': { '04': 213.2, '05': 280.5, '5': 353.4, '6': 403.9, '8': 516.1, '10': 594.7, '12': 673.2 },
  },
  '33': {
    '2': { '04': 289.2, '05': 380.5, '5': 547.9, '6': 662.1, '8': 783.8, '10': 821.9, '12': 0 },
    '3.5': { '04': 263.7, '05': 347, '5': 437.2, '6': 499.7, '8': 638.5, '10': 735.6, '12': 832.8 },
  },
  '36': {
    '3': { '04': 328.7, '05': 432.5, '5': 622.8, '6': 804.4, '8': 942.8, '10': 934.2, '12': 0 },
    '4': { '04': 310.5, '05': 408.5, '5': 514.7, '6': 588.2, '8': 751.6, '10': 866, '12': 980.4 },
  },
  '39': {
    '3': { '04': 391.4, '05': 515.8, '5': 741.6, '6': 957.9, '8': 1123, '10': 1112, '12': 0 },
    '4': { '04': 370.9, '05': 488, '5': 614.9, '6': 702.7, '8': 897.9, '10': 1035, '12': 1171 },
  },
})

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
    const diameter = (props.queryForm.diameter || 0).toString()
    const pitch = (props.queryForm.pitch || 0).toString()
    const intensityKey = (formulasForm.value.intensity || '').toString()
    // console.log(diameter, pitch, intensityKey, JSON.stringify(safeVaule.value))
    result.value = safeVaule.value[diameter]?.[pitch]?.[intensityKey] || 0
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
