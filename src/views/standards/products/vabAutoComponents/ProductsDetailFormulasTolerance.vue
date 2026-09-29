<!--
 * @Author: ${git_name}
 * @Date: 2025-05-09 18:18:43
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-20 18:23:41
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasTolerance.vue
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
      <el-form-item label="输入">
        <el-input v-model="formulasForm.size" clearable style="width: 280px">
          <template #prepend>尺寸</template>
          <template #append>{{ metric }}</template>
        </el-input>
      </el-form-item>
    </el-form>
    <el-form class="demo-form-inline" :inline="true">
      <el-form-item label="可选">
        <el-select v-model="formulasForm.hole" clearable style="width: 120px" @change="changeHole">
          <template #prefix>孔类</template>
          <el-option v-for="item in state.hole" :key="`hole[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item label="或者">
        <el-select v-model="formulasForm.axis" clearable style="width: 160px" @change="changeAxis">
          <template #prefix>轴类</template>
          <el-option v-for="item in state.axis" :key="`axis[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
    </el-form>
    <el-form class="demo-form-inline" :inline="true">
      <el-form-item label="公差等级">
        <el-select v-model="formulasForm.grade" placeholder="" style="width: 80px">
          <template #prefix>IT</template>
          <el-option v-for="item in state.grade" :key="`grade[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="changeResult">计算</el-button>
      </el-form-item>
      <el-form-item label="=" label-width="5px">
        <el-text v-if="formulasForm.hole != '' || formulasForm.axis != ''" style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(resultMin || 0) }} ~ {{ formatNumber(resultMax || 0) }}
        </el-text>
        <el-text v-else style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(result || 0) }}
        </el-text>
        <el-icon
          v-if="formulasForm.hole != '' || formulasForm.axis != ''"
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="handleCopy(`${formatNumber(resultMin)}, ${formatNumber(resultMax)}`, currentFormula.name)"
        >
          <document-copy />
        </el-icon>
        <el-icon
          v-else
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
  grade: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
  axis: ['a', 'b', 'c', 'cd', 'd', 'e', 'ef', 'f', 'fg', 'g', 'h', 'js', 'm', 'n', 'p', 'r', 's', 't', 'u', 'v', 'x', 'y', 'z'],
  hole: ['A', 'B', 'C', 'CD', 'D', 'E', 'EF', 'F', 'FG', 'G', 'H', 'JS', 'P', 'R', 'S', 'T', 'U', 'V', 'X', 'Y', 'Z'],
  defaultFormulasForm: {
    size: '',
    grade: '',
    axis: '',
    hole: '',
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
// 孔改变
const changeHole = () => {
  if (!!formulasForm.value.hole) {
    formulasForm.value.axis = ''
  }
}
// 轴改变
const changeAxis = () => {
  if (!!formulasForm.value.axis) {
    formulasForm.value.hole = ''
  }
}

// 计算全部公式
const changeResult = () => {
  if (state.submit) return
  state.submit = true
  result.value = 0
  resultMin.value = 0
  resultMax.value = 0
  if (!formulasForm.value.size || !formulasForm.value.grade) return

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
      .replace(/\{x\}/g, variables)

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
    let standardObj = {} as any
    Object.keys(tolerance.value['standard'] || {}).forEach((key) => {
      if (evaluateCondition(key, formulasForm.value.size)) {
        standardObj = tolerance.value['standard'][key]
      }
    })
    const deviation = standardObj[`IT${formulasForm.value.grade}`] || 0
    result.value = Number(deviation)
    // 孔存在的时候
    if (!!formulasForm.value.hole) {
      let holeObj = {} as any
      Object.keys(tolerance.value['hole'] || {}).forEach((key) => {
        if (evaluateCondition(key, formulasForm.value.size)) {
          holeObj = tolerance.value['hole'][key]
        }
      })
      if (['A', 'B', 'C', 'CD', 'D', 'E', 'EF', 'F', 'FG', 'G', 'H', 'JS'].includes(formulasForm.value.hole)) {
        // 下偏差：A	B	C	CD	D	E	EF	F	FG	G	H	JS
        if (formulasForm.value.hole == 'JS') {
          resultMin.value = getMathEvaluate(`${formulasForm.value.size}-(${deviation}/2)`, {})
          resultMax.value = getMathEvaluate(`${formulasForm.value.size}+(${deviation}/2)`, {})
          // console.log(`${formulasForm.value.size}-(${deviation}/2)`, `${formulasForm.value.size}+(${deviation}/2)`)
        } else {
          const bottomDeviation = `${Number(holeObj[formulasForm.value.hole] || 0) >= 0 ? '+' : ''}${holeObj[formulasForm.value.hole] || 0}`
          const topDeviation = `${bottomDeviation}+${deviation}`
          resultMin.value = getMathEvaluate(`${formulasForm.value.size}${bottomDeviation}`, {})
          resultMax.value = getMathEvaluate(`${formulasForm.value.size}${topDeviation}`, {})
          // console.log(`${formulasForm.value.size}${bottomDeviation}`, `${formulasForm.value.size}${topDeviation}`)
        }
      } else if (['P', 'R', 'S', 'T', 'U', 'V', 'X', 'Y', 'Z'].includes(formulasForm.value.hole)) {
        // 上偏差：P	R	S	T	U	V	X	Y	Z
        const topDeviation = `${Number(holeObj[formulasForm.value.hole] || 0) >= 0 ? '+' : ''}${holeObj[formulasForm.value.hole] || 0}`
        const bottomDeviation = `${topDeviation}-${deviation}`
        resultMin.value = getMathEvaluate(`${formulasForm.value.size}${bottomDeviation}`, {})
        resultMax.value = getMathEvaluate(`${formulasForm.value.size}${topDeviation}`, {})
        // console.log(`${formulasForm.value.size}${bottomDeviation}`, `${formulasForm.value.size}${topDeviation}`)
      }
    }
    // 轴存在的时候
    if (!!formulasForm.value.axis) {
      let axisObj = {} as any
      Object.keys(tolerance.value['axis'] || {}).forEach((key) => {
        if (evaluateCondition(key, formulasForm.value.size)) {
          axisObj = tolerance.value['axis'][key]
        }
      })
      if (['a', 'b', 'c', 'cd', 'd', 'e', 'ef', 'f', 'fg', 'g', 'h', 'js'].includes(formulasForm.value.axis)) {
        // 上偏差：a	b	c	cd	d	e	ef	f	fg	g	h	js
        if (formulasForm.value.axis == 'js') {
          resultMin.value = getMathEvaluate(`${formulasForm.value.size}-(${deviation}/2)`, {})
          resultMax.value = getMathEvaluate(`${formulasForm.value.size}+(${deviation}/2)`, {})
        } else {
          const topDeviation = `${Number(axisObj[formulasForm.value.axis] || 0) >= 0 ? '+' : ''}${axisObj[formulasForm.value.axis] || 0}`
          const bottomDeviation = `${topDeviation}-${deviation}`
          resultMin.value = getMathEvaluate(`${formulasForm.value.size}${bottomDeviation}`, {})
          resultMax.value = getMathEvaluate(`${formulasForm.value.size}${topDeviation}`, {})
          // console.log(`${formulasForm.value.size}${bottomDeviation}`, `${formulasForm.value.size}${topDeviation}`)
        }
      } else if (['m', 'n', 'p', 'r', 's', 't', 'u', 'v', 'x', 'y', 'z'].includes(formulasForm.value.axis)) {
        // 下偏差：m	n	p	r	s	t	u	v	x	y	z
        const bottomDeviation = `${Number(axisObj[formulasForm.value.axis] || 0) >= 0 ? '+' : ''}${axisObj[formulasForm.value.axis] || 0}`
        const topDeviation = `${bottomDeviation}+${deviation}`
        resultMin.value = getMathEvaluate(`${formulasForm.value.size}${bottomDeviation}`, {})
        resultMax.value = getMathEvaluate(`${formulasForm.value.size}${topDeviation}`, {})
        // console.log(`${formulasForm.value.size}${bottomDeviation}`, `${formulasForm.value.size}${topDeviation}`)
      }
    }
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
  resultMin.value = 0
  resultMax.value = 0
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })
</script>

<style lang="scss" scoped></style>
