<!--
 * @Author: ${git_name}
 * @Date: 2025-05-08 13:35:37
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-25 17:17:25
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasAssemblyTorque.vue
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
        <el-select v-model="formulasForm.intensity" placeholder="" style="width: 150px">
          <template #prefix>强度等级</template>
          <el-option v-for="item in state.intensity" :key="`intensity[${item.value}]`" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
    </el-form>
    <el-form class="demo-form-inline" :inline="true">
      <el-alert :closable="false" show-icon type="info">
        <p>请从下面"材料"+"表面处理"+"润滑"选择后进行计算得到装配扭矩最小值和最大值，也可以直接输入“K”进行计算最小值</p>
      </el-alert>
      <el-form-item label="可选">
        <el-select v-model="formulasForm.material" style="width: 120px" @change="changeMaterial">
          <template #prefix>材料</template>
          <el-option v-for="item in state.material" :key="`material[${item.value}]`" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-select v-model="formulasForm.exteriors" style="width: 160px" @change="changeExteriors">
          <template #prefix>表面处理</template>
          <el-option v-for="item in state.exteriors" :key="`exteriors[${item}]`" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-popover :offset="8" placement="right" :show-after="300" title="润滑说明" trigger="hover" :width="400">
          <template #reference>
            <el-select v-model="formulasForm.lubricate" style="width: 160px" @change="changeLubricate">
              <template #prefix>润滑</template>
              <el-option v-for="item in state.lubricate" :key="`lubricate[${item}]`" :label="item" :value="item" />
            </el-select>
          </template>
          <template #default>
            <p><b>1、专业润滑：</b></p>
            <p>固体润滑油，如在润滑清漆或软膏中添加MoS2、石墨、聚四氟乙烯、PA、PE、PI，液化蜡等</p>
            <p><b>2、一般润滑：</b></p>
            <p>MoS2、石墨、液化蜡等</p>
            <p><b>3、防松胶：</b></p>
            <p>如螺纹部分涂的防松胶</p>
          </template>
        </el-popover>
      </el-form-item>
    </el-form>
    <el-form class="demo-form-inline" :inline="true">
      <el-form-item label="或输入">
        <el-popover
          placement="top"
          popper-style="box-shadow: rgb(14 18 22 / 35%) 0px 10px 38px -10px, rgb(14 18 22 / 20%) 0px 10px 20px -15px; padding: 20px;"
          title="K输入建议"
          :width="600"
        >
          <template #reference>
            <el-input v-model="formulasForm.k" clearable style="width: 240px">
              <template #prepend>K</template>
            </el-input>
          </template>
          <template #default>
            <el-table :border="true" :data="tipData" :stripe="true" style="width: 100%">
              <el-table-column label="配套零件" property="remarks" />
              <el-table-column label="K" property="recommend" width="100" />
            </el-table>
          </template>
        </el-popover>
      </el-form-item>
    </el-form>
    <el-form class="demo-form-inline" :inline="true">
      <el-form-item>
        <el-button type="primary" @click="changeResult">计算</el-button>
      </el-form-item>
      <el-form-item label="=" label-width="5px">
        <el-text v-if="formulasForm.k == ''" style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(resultMin || 0) }} ~ {{ formatNumber(resultMax || 0) }}
        </el-text>
        <el-text v-else style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(result || 0) }}
        </el-text>
        <el-icon
          v-if="formulasForm.k == ''"
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="handleCopy(`${formatNumber(resultMin || 0)}, ${formatNumber(resultMax || 0)}`, currentFormula.name)"
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
          active-text="N*m"
          active-value="N*m"
          inactive-text="kg*f*cm"
          inactive-value="kg*f*cm"
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
  material: [],
  exteriors: [],
  lubricate: [],
  defaultFormulasForm: {
    intensity: '',
    material: '',
    exteriors: '',
    lubricate: '',
    k: '',
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

const kOptin = ref<any>({
  碳钢: {
    本色: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 一般润滑: { kMin: 0.14, kMax: 0.24 } },
    发黑: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 涂油: { kMin: 0.14, kMax: 0.24 } },
    蓝白锌: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 防松胶: { kMin: 0.14, kMax: 0.24 }, 无润滑: { kMin: 0.2, kMax: 0.35 } },
    彩锌: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 防松胶: { kMin: 0.14, kMax: 0.24 }, 无润滑: { kMin: 0.2, kMax: 0.35 } },
    镀锌铁: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 防松胶: { kMin: 0.14, kMax: 0.24 }, 无润滑: { kMin: 0.2, kMax: 0.35 } },
    镀锌镍: { 专业润滑: { kMin: 0.04, kMax: 0.16 }, 防松胶: { kMin: 0.14, kMax: 0.24 }, 无润滑: { kMin: 0.3, kMax: null } },
    热浸锌: { 专业润滑: { kMin: 0.08, kMax: 0.16 }, 无润滑: { kMin: 0.2, kMax: 0.35 } },
    特氟龙: { 一般润滑: { kMin: 0.08, kMax: 0.16 } },
  },
  铝: { 本色: { 专业润滑: { kMin: 0.08, kMax: 0.16 }, 无润滑: { kMin: 0.3, kMax: null } } },
  不锈钢: {
    本色: {
      专业润滑: { kMin: 0.08, kMax: 0.16 },
      一般润滑: { kMin: 0.14, kMax: 0.24 },
      涂油: { kMin: 0.2, kMax: 0.35 },
      无润滑: { kMin: 0.3, kMax: null },
    },
  },
})

const formulasForm = ref<any>(JSON.parse(JSON.stringify(state.defaultFormulasForm)))
const metric = ref(props.currentFormula?.unit || 'N*m')
const result = ref(0)
const resultMin = ref(0)
const resultMax = ref(0)

const tipData = ref([
  {
    remarks: '干燥、清洁，有一层薄油膜（已润滑）',
    recommend: '0.15~0.20',
  },
  {
    remarks: '油、蜡或不同镀层或硬垫圈的附加润滑涂层（已润滑）',
    recommend: '0.10~0.15',
  },
  {
    remarks: '螺纹和头部轴承表面覆盖有高性能润滑剂或抗咬合化合物（润滑）',
    recommend: '0.05',
  },
  {
    remarks: '某些材料的组合，例如奥氏体不锈钢螺钉/螺栓和未润滑或涂层（干燥）的零件',
    recommend: '0.35',
  },
])
// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}
const changeUnit = () => {
  const loadFactor = 9.8
  if (metric.value == 'N*m') {
    resultMin.value = resultMin.value / loadFactor
    resultMax.value = resultMax.value / loadFactor
    result.value = result.value / loadFactor
  } else if (metric.value == 'kg*f*cm') {
    resultMin.value = resultMin.value * loadFactor
    resultMax.value = resultMax.value * loadFactor
    result.value = result.value * loadFactor
  }
}
// 材料变化
const changeMaterial = () => {
  formulasForm.value.exteriors = ''
  state.exteriors = []
  if (kOptin.value?.[formulasForm.value.material]) {
    state.exteriors = Object.keys(kOptin.value[formulasForm.value.material])
  }
}
// 表面处理变化
const changeExteriors = () => {
  formulasForm.value.lubricate = ''
  state.lubricate = []
  if (kOptin.value?.[formulasForm.value.material]?.[formulasForm.value.exteriors]) {
    state.lubricate = Object.keys(kOptin.value?.[formulasForm.value.material]?.[formulasForm.value.exteriors])
  }
}
// 润滑变化
const changeLubricate = () => {}

// 计算全部公式
const changeResult = () => {
  if (state.submit) return
  state.submit = true
  result.value = 0
  resultMin.value = 0
  resultMax.value = 0
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

    const kValue = kOptin.value?.[formulasForm.value.material]?.[formulasForm.value.exteriors]?.[formulasForm.value.lubricate]

    // 计算固定公式
    if (!!formulasForm.value.k) {
      const columnar = props.currentFormula?.columnar.replace('{K}', formulasForm.value.k)
      result.value = evaluateFormula(columnar, props.formulas, variables)
    } else if (kValue) {
      if (kValue['kMin']) {
        const columnar = props.currentFormula?.columnar.replace('{K}', kValue['kMin'])
        resultMin.value = evaluateFormula(columnar, props.formulas, variables)
      }
      if (kValue['kMax']) {
        const columnar = props.currentFormula?.columnar.replace('{K}', kValue['kMax'])
        resultMax.value = evaluateFormula(columnar, props.formulas, variables)
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
  state.material = []
  state.intensity = []
  Object.entries(material.value || {}).forEach(([key, value]) => {
    state.material.push({
      value: value,
      label: value as string,
    })
  })
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
