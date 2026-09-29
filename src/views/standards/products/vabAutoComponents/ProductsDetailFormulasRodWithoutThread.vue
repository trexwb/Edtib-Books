<!--
 * @Author: ${git_name}
 * @Date: 2025-05-11 14:05:57
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-25 17:08:48
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasRodWithoutThread.vue
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
      <el-form-item label="螺纹">
        <el-select v-model="formulasForm.grade" clearable style="width: 120px">
          <template #prefix>等级</template>
          <el-option v-for="item in state.externalGrade" :key="`externalGrade[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item label="">
        <el-select v-model="formulasForm.deviation" clearable style="width: 120px">
          <template #prefix>偏差</template>
          <el-option v-for="item in state.externalDeviation" :key="`externalDeviation[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="changeResult">计算</el-button>
      </el-form-item>
      <el-form-item v-if="resultMin != resultMax" label="=" label-width="5px">
        <el-text style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(resultMin || 0) }} ~ {{ formatNumber(resultMax || 0) }}
        </el-text>
        <el-icon
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="handleCopy(`${formatNumber(resultMin || 0)}, ${formatNumber(resultMax || 0)}`, currentFormula.name)"
        >
          <document-copy />
        </el-icon>
      </el-form-item>
      <el-form-item v-else-if="resultMin && resultMax" label="=" label-width="5px">
        <el-text style="font-size: 28px; font-weight: bold" type="primary">无结果（请确认螺距和螺纹公差等级）</el-text>
      </el-form-item>
      <el-form-item v-if="resultMin != resultMax">
        <template #label>
          <div>结果换算:</div>
        </template>
        <el-switch
          v-model="metric"
          active-text="mm"
          active-value="mm"
          inactive-text="inch"
          inactive-value="inch"
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
import * as math from 'mathjs'
import { getMathEvaluate, formatNumber } from '/@/utils/evaluateCondition'
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
  externalGrade: [4, 6, 8],
  externalDeviation: ['e', 'f', 'g', 'h'],
  defaultFormulasForm: {
    grade: '6',
    deviation: 'g',
  },
})

// 螺纹基本偏差
const externalDeviation = ref<any>({
  '0.2': { g: '-0.017', h: '0' },
  '0.25': { g: '-0.018', h: '0' },
  '0.3': { g: '-0.018', h: '0' },
  '0.35': { f: '-0.034', g: '-0.019', h: '0' },
  '0.4': { f: '-0.034', g: '-0.02', h: '0' },
  '0.45': { f: '-0.035', g: '-0.02', h: '0' },
  '0.5': { e: '-0.05', f: '-0.036', g: '-0.02', h: '0' },
  '0.6': { e: '-0.053', f: '-0.036', g: '-0.021', h: '0' },
  '0.7': { e: '-0.056', f: '-0.038', g: '-0.022', h: '0' },
  '0.75': { e: '-0.056', f: '-0.038', g: '-0.022', h: '0' },
  '0.8': { e: '-0.06', f: '-0.038', g: '-0.024', h: '0' },
  '1': { e: '-0.06', f: '-0.04', g: '-0.026', h: '0' },
  '1.25': { e: '-0.063', f: '-0.042', g: '-0.028', h: '0' },
  '1.5': { e: '-0.067', f: '-0.045', g: '-0.032', h: '0' },
  '1.75': { e: '-0.071', f: '-0.048', g: '-0.034', h: '0' },
  '2': { e: '-0.071', f: '-0.052', g: '-0.038', h: '0' },
  '2.5': { e: '-0.08', f: '-0.058', g: '-0.042', h: '0' },
  '3': { e: '-0.085', f: '-0.063', g: '-0.048', h: '0' },
  '3.5': { e: '-0.09', f: '-0.07', g: '-0.053', h: '0' },
  '4': { e: '-0.095', f: '-0.075', g: '-0.06', h: '0' },
  '4.5': { e: '-0.1', f: '-0.08', g: '-0.063', h: '0' },
  '5': { e: '-0.106', f: '-0.085', g: '-0.071', h: '0' },
  '5.5': { e: '-0.112', f: '-0.09', g: '-0.075', h: '0' },
  '6': { e: '-0.118', f: '-0.095', g: '-0.08', h: '0' },
  '8': { e: '-0.14', f: '-0.118', g: '-0.1', h: '0' },
})
// 外螺纹大径公差Td
const externalBig = ref<any>({
  '0.2': { '4': '0.036', '6': '0.056' },
  '0.25': { '4': '0.042', '6': '0.067' },
  '0.3': { '4': '0.048', '6': '0.075' },
  '0.35': { '4': '0.053', '6': '0.085' },
  '0.4': { '4': '0.06', '6': '0.095' },
  '0.45': { '4': '0.063', '6': '0.1' },
  '0.5': { '4': '0.067', '6': '0.106' },
  '0.6': { '4': '0.08', '6': '0.125' },
  '0.7': { '4': '0.09', '6': '0.14' },
  '0.75': { '4': '0.09', '6': '0.14' },
  '0.8': { '4': '0.095', '6': '0.15', '8': '0.236' },
  '1': { '4': '0.112', '6': '0.18', '8': '0.28' },
  '1.25': { '4': '0.132', '6': '0.212', '8': '0.335' },
  '1.5': { '4': '0.15', '6': '0.236', '8': '0.375' },
  '1.75': { '4': '0.17', '6': '0.265', '8': '0.425' },
  '2': { '4': '0.18', '6': '0.28', '8': '0.45' },
  '2.5': { '4': '0.212', '6': '0.335', '8': '0.53' },
  '3': { '4': '0.236', '6': '0.375', '8': '0.6' },
  '3.5': { '4': '0.265', '6': '0.425', '8': '0.67' },
  '4': { '4': '0.3', '6': '0.475', '8': '0.75' },
  '4.5': { '4': '0.315', '6': '0.5', '8': '0.8' },
  '5': { '4': '0.335', '6': '0.53', '8': '0.85' },
  '5.5': { '4': '0.355', '6': '0.56', '8': '0.9' },
  '6': { '4': '0.375', '6': '0.6', '8': '0.95' },
  '8': { '4': '0.45', '6': '0.71', '8': '1.18' },
})
// 外螺纹中径公差
const externalMiddle = ref<any>({
  '0.99<{d}&{d}≤1.4': {
    '0.2': { '3': '0.024', '4': '0.03', '5': '0.038', '6': '0.048' },
    '0.25': { '3': '0.026', '4': '0.034', '5': '0.042', '6': '0.053' },
    '0.3': { '3': '0.028', '4': '0.036', '5': '0.045', '6': '0.056' },
  },
  '1.4<{d}&{d}≤2.8': {
    '0.2': { '3': '0.025', '4': '0.032', '5': '0.04', '6': '0.05' },
    '0.25': { '3': '0.028', '4': '0.036', '5': '0.045', '6': '0.056' },
    '0.35': { '3': '0.032', '4': '0.04', '5': '0.05', '6': '0.063', '7': '0.08' },
    '0.4': { '3': '0.034', '4': '0.042', '5': '0.053', '6': '0.067', '7': '0.085' },
    '0.45': { '3': '0.036', '4': '0.045', '5': '0.056', '6': '0.071', '7': '0.09' },
  },
  '2.8<{d}&{d}≤5.6': {
    '0.35': { '3': '0.034', '4': '0.042', '5': '0.053', '6': '0.067', '7': '0.085' },
    '0.5': { '3': '0.038', '4': '0.048', '5': '0.06', '6': '0.075', '7': '0.095' },
    '0.6': { '3': '0.042', '4': '0.053', '5': '0.067', '6': '0.085', '7': '0.106' },
    '0.7': { '3': '0.045', '4': '0.056', '5': '0.071', '6': '0.09', '7': '0.112' },
    '0.75': { '3': '0.045', '4': '0.056', '5': '0.071', '6': '0.09', '7': '0.112' },
    '0.8': { '3': '0.048', '4': '0.6', '5': '0.75', '6': '0.095', '7': '0.118', '8': '0.15', '9': '0.19' },
  },
  '5.6<{d}&{d}≤11.2': {
    '1': { '3': '0.056', '4': '0.071', '5': '0.09', '6': '0.112', '7': '0.14', '8': '0.18', '9': '0.224' },
    '0.75': { '3': '0.05', '4': '0.063', '5': '0.08', '6': '0.1', '7': '0.125' },
    '1.25': { '3': '0.06', '4': '0.075', '5': '0.095', '6': '0.118', '7': '0.15', '8': '0.19', '9': '0.236' },
    '1.5': { '3': '0.067', '4': '0.085', '5': '0.106', '6': '0.132', '7': '0.17', '8': '0.212', '9': '0.265' },
  },
  '11.2<{d}&{d}≤22.4': {
    '1': { '3': '0.06', '4': '0.075', '5': '0.095', '6': '0.118', '7': '0.15', '8': '0.19', '9': '0.236' },
    '2': { '3': '0.08', '4': '0.1', '5': '0.125', '6': '0.16', '7': '0.2', '8': '0.25', '9': '0.315' },
    '1.25': { '3': '0.067', '4': '0.085', '5': '0.106', '6': '0.132', '7': '0.17', '8': '0.212', '9': '0.265' },
    '1.5': { '3': '0.071', '4': '0.09', '5': '0.112', '6': '0.14', '7': '0.18', '8': '0.224', '9': '0.28' },
    '1.75': { '3': '0.075', '4': '0.095', '5': '0.118', '6': '0.15', '7': '0.19', '8': '0.236', '9': '0.3' },
    '2.5': { '3': '0.085', '4': '0.106', '5': '0.132', '6': '0.17', '7': '0.212', '8': '0.265', '9': '0.335' },
  },
  '22.4<{d}&{d}≤45': {
    '1': { '3': '0.063', '4': '0.08', '5': '0.1', '6': '0.125', '7': '0.16', '8': '0.2', '9': '0.25' },
    '2': { '3': '0.085', '4': '0.106', '5': '0.132', '6': '0.17', '7': '0.212', '8': '0.265', '9': '0.335' },
    '3': { '3': '0.1', '4': '0.125', '5': '0.16', '6': '0.2', '7': '0.25', '8': '0.315', '9': '0.4' },
    '4': { '3': '0.112', '4': '0.14', '5': '0.18', '6': '0.224', '7': '0.28', '8': '0.355', '9': '0.45' },
    '1.5': { '3': '0.075', '4': '0.095', '5': '0.118', '6': '0.15', '7': '0.19', '8': '0.236', '9': '0.3' },
    '3.5': { '3': '0.106', '4': '0.132', '5': '0.17', '6': '0.212', '7': '0.265', '8': '0.335', '9': '0.425' },
    '4.5': { '3': '0.118', '4': '0.15', '5': '0.19', '6': '0.236', '7': '0.3', '8': '0.375', '9': '0.475' },
  },
  '45<{d}&{d}≤90': {
    '2': { '3': '0.09', '4': '0.112', '5': '0.14', '6': '0.18', '7': '0.224', '8': '0.28', '9': '0.355' },
    '3': { '3': '0.106', '4': '0.132', '5': '0.17', '6': '0.212', '7': '0.265', '8': '0.335', '9': '0.425' },
    '4': { '3': '0.118', '4': '0.15', '5': '0.19', '6': '0.236', '7': '0.3', '8': '0.375', '9': '0.475' },
    '5': { '3': '0.125', '4': '0.16', '5': '0.2', '6': '0.25', '7': '0.315', '8': '0.4', '9': '0.5' },
    '6': { '3': '0.14', '4': '0.18', '5': '0.224', '6': '0.28', '7': '0.355', '8': '0.45', '9': '0.56' },
    '1.5': { '3': '0.08', '4': '0.1', '5': '0.125', '6': '0.16', '7': '0.2', '8': '0.25', '9': '0.315' },
    '5.5': { '3': '0.132', '4': '0.17', '5': '0.212', '6': '0.265', '7': '0.335', '8': '0.425', '9': '0.53' },
  },
  '90<{d}&{d}≤180': {
    '2': { '3': '0.095', '4': '0.118', '5': '0.15', '6': '0.19', '7': '0.236', '8': '0.3', '9': '0.375' },
    '3': { '3': '0.112', '4': '0.14', '5': '0.18', '6': '0.224', '7': '0.28', '8': '0.355', '9': '0.45' },
    '4': { '3': '0.125', '4': '0.16', '5': '0.2', '6': '0.25', '7': '0.315', '8': '0.4', '9': '0.5' },
    '6': { '3': '0.15', '4': '0.19', '5': '0.236', '6': '0.3', '7': '0.375', '8': '0.475', '9': '0.6' },
    '8': { '3': '0.17', '4': '0.212', '5': '0.265', '6': '0.335', '7': '0.425', '8': '0.53', '9': '0.67' },
  },
  '180<{d}&{d}≤355': {
    '3': { '3': '0.125', '4': '0.16', '5': '0.2', '6': '0.25', '7': '0.315', '8': '0.4', '9': '0.5' },
    '4': { '3': '0.14', '4': '0.18', '5': '0.224', '6': '0.28', '7': '0.355', '8': '0.45', '9': '0.56' },
    '6': { '3': '0.16', '4': '0.2', '5': '0.25', '6': '0.315', '7': '0.4', '8': '0.5', '9': '0.63' },
    '8': { '3': '0.18', '4': '0.224', '5': '0.28', '6': '0.355', '7': '0.45', '8': '0.56', '9': '0.71' },
  },
})

const formulasForm = ref<any>(JSON.parse(JSON.stringify(state.defaultFormulasForm)))
const metric = ref(props.currentFormula?.unit || 'mm')
const result = ref(0)
const resultMin = ref(0)
const resultMax = ref(0)
// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}
const changeUnit = () => {
  const lengthFactor = 25.4
  if (metric.value == 'inch') {
    result.value = result.value / lengthFactor
    resultMin.value = resultMin.value / lengthFactor
    resultMax.value = resultMax.value / lengthFactor
  } else if (metric.value == 'mm') {
    result.value = result.value * lengthFactor
    resultMin.value = resultMin.value * lengthFactor
    resultMax.value = resultMax.value * lengthFactor
  }
}

// 计算全部公式
const changeResult = () => {
  if (state.submit) return
  state.submit = true
  // result.value = 0;
  if (!formulasForm.value.deviation || !formulasForm.value.grade) return
  const variables = {} as any
  variables['P'] = Number(props.queryForm.pitch || 0)
  variables['d'] = Number(props.queryForm.diameter || 0)

  function toHalfWidth(str: string) {
    return str
      .replace(/[\uff10-\uff5e]/g, function (char) {
        return String.fromCharCode(char.charCodeAt(0) - 65248)
      })
      .replace(/\u3000/g, ' ') // 全角空格转换为半角空格
  }
  function evaluateCondition(condition: string, variables: any) {
    const parsedCondition = condition
      .replace(/≤/g, '<=')
      .replace(/≥/g, '>=')
      .replace(/&/g, '&&')
      .replace(/\|/g, '||')
      .replace(/./g, (char) => toHalfWidth(char))
      .replace(/\{d\}/g, variables['d'] || 0)
      .replace(/\{P\}/g, variables['P'] || 0)
    try {
      // C-B2: 使用 mathjs 受控求值替代 eval（mathjs 不识别 &&/||/!，先翻译为 and/or/not），杜绝任意代码执行
      const mathExpr = parsedCondition
        .replace(/&&/g, ' and ')
        .replace(/\|\|/g, ' or ')
        .replace(/!(?!=)/g, ' not ')
      return !!math.evaluate(mathExpr, {})
    } catch (error) {
      console.error('Error evaluating condition:', parsedCondition, error)
      return false
    }
  }
  try {
    // 外螺纹
    const deviation = externalDeviation.value[variables['P'].toString()]?.[formulasForm.value.deviation] || 0
    const deviationBig = externalBig.value[variables['P'].toString()]?.[formulasForm.value.grade.toString()] || 0

    let middleObj = {} as any
    Object.keys(externalMiddle.value || {}).forEach((key) => {
      if (evaluateCondition(key, variables)) {
        middleObj = externalMiddle.value[key]
      }
    })
    const middleMin = middleObj[variables['P'].toString()]?.[formulasForm.value.grade.toString()] || 0
    result.value = getMathEvaluate(`d-0.6495*P${deviation >= 0 ? '+' : ''}${deviation}-${middleMin}`, variables)
    // 最小值
    resultMin.value = getMathEvaluate(`${result.value}+0.3*${middleMin}`, variables)
    // 最大值
    resultMax.value = getMathEvaluate(`${result.value}+0.7*${middleMin}`, variables)
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
  formulasForm.value.grade = props.productRow.shapes_id.includes(2) ? '6' : '6'
  formulasForm.value.deviation = props.productRow.shapes_id.includes(2) ? 'g' : 'G'
  result.value = 0
  resultMin.value = 0
  resultMax.value = 0
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })
</script>

<style lang="scss" scoped></style>
