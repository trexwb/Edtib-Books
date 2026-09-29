<!--
 * @Author: ${git_name}
 * @Date: 2025-04-29 14:12:25
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-15 15:11:45
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulas.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-card>
    <template #header>
      <div class="card-header">
        <div>计算公式</div>
        <el-tooltip effect="light" :offset="0" placement="left">
          <template #content>
            <div style="width: 650px">
              <p>
                Edtib提供的所有计算工具、公式及数据结果仅供参考，旨在为用户提供一定的技术与工程支持。尽管我们力求提供准确且可靠的计算方法，但这些工具和公式并不保证在所有情况下均适用，也不代表最终结论。
              </p>
              <p>
                用户在使用Edtib提供的任何功能时，应自行判断其适用性，并对其使用后果承担全部责任。因用户依据本工具所提供的信息进行操作而引发的任何直接或间接损失，包括但不限于经济损失、工程事故或其他责任纠纷，本公司概不负责。
              </p>
              <p>如需专业的工程指导或技术支持，请联系：13216118255。</p>
              <p>感谢您的理解与支持！</p>
            </div>
          </template>
          <el-icon>
            <info-filled />
          </el-icon>
        </el-tooltip>
      </div>
    </template>
    <el-space style="margin-bottom: 14px" wrap>
      <!-- v-if="conditionValidate(productRow.shapes_id, item?.condition?.shapesIds) &&
      conditionValidate(productRow.categories_id, item?.condition?.categoriesIds)" -->
      <!-- 后台公式 -->
      <span v-for="(item, index) in formulasActive" :key="`formulasButton[${index}]`">
        <el-button :type="`${state.formulas !== item.code ? 'primary' : 'info'}`" @click="handelFormulas(item.code)">
          {{ item.name }}
        </el-button>
      </span>
      <!-- 固定显示公式 -->
      <template v-for="(item, index) in fixedFormula" :key="`formulasButton[${index}]`">
        <span v-if="index === 'volume' ? shapeVolumeReady : conditionShow(item)">
          <el-button v-if="conditionShow(item)" :type="`${state.formulas !== index ? 'primary' : 'info'}`" @click="handelFormulas(index)">
            <span v-if="index === 'hardnessToStrength'">
              硬度
              <el-icon>
                <switch />
              </el-icon>
              强度
            </span>
            <span v-else>{{ item.name }}</span>
          </el-button>
        </span>
      </template>
    </el-space>
  </vab-card>
  <div v-for="(item, index) in formulasActive" v-show="state.formulas == item.code" :key="`formulasActive[${index}]`">
    <!-- 千支重组件 -->
    <products-detail-formulas-thousand-weight
      v-if="item.code == 'thousandWeight' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[item.code] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 通用公式显示 -->
    <products-detail-formulas-common
      v-else-if="conditionShow(item)"
      :ref="(el: any) => (formulasRef[item.code] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
  </div>
  <!-- 固定显示公式 -->
  <div ref="formulasShowRefs"></div>
  <div v-for="(item, index) in fixedFormula" v-show="state.formulas == index" :key="`fixedFormula[${index}]`">
    <!-- 体积（关联形状体积公式求和） -->
    <products-detail-formulas-volume
      v-if="index == 'volume' && shapeVolumeReady"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      :shape-parts="shapeParts"
      @close="handelCleanFormulas"
    />
    <!-- 最小破话扭矩 -->
    <products-detail-formulas-min-breaking-torque
      v-else-if="index == 'minBreakingTorque' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 装配扭矩 -->
    <products-detail-formulas-assembly-torque
      v-else-if="index == 'assemblyTorque' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 抗剪力 -->
    <products-detail-formulas-shear-load
      v-else-if="index == 'shearLoad' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 抗拉力 -->
    <products-detail-formulas-tensile-load
      v-else-if="index == 'tensileLoad' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 螺母保证载荷（安全载荷） -->
    <products-detail-formulas-safe-load
      v-else-if="index == 'safeLoad' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 硬度⇄强度 -->
    <products-detail-formulas-hardness-to-strength
      v-else-if="index == 'hardnessToStrength' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 公差计算 -->
    <products-detail-formulas-tolerance
      v-else-if="index == 'tolerance' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 螺纹公差 -->
    <products-detail-formulas-thread-tolerance
      v-else-if="index == 'threadTolerance' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 螺纹毛坯 -->
    <products-detail-formulas-rod-without-thread
      v-else-if="index == 'rodWithoutThread' && conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
    <!-- 通用公式显示 -->
    <products-detail-formulas-common
      v-else-if="conditionShow(item)"
      :ref="(el: any) => (formulasRef[index] = el)"
      :current-formula="item"
      :formulas="formulas"
      :product-row="productRow"
      :query-form="queryForm"
      @close="handelCleanFormulas"
    />
  </div>
</template>

<script setup lang="ts">
import { InfoFilled, Switch } from '@element-plus/icons-vue'
import { useAclStore } from '/@/store/modules/acl'
import { evaluateConditionDefault } from '/@/utils/evaluateCondition'

const templateName = 'StandardsProductsDetailFormulas'
/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { languages, defaultItem, tolerance, intensity, material } = storeToRefs(aclStore) // 解构多语言列表响应式引用

const formulasRef: Record<string, any> = {} // 编辑器实例的引用集合
const formulasShowRefs = ref<any>(null)

defineOptions({
  name: templateName,
})

// 定义 props 类型
const props = defineProps(['productRow', 'variables', 'formulas', 'queryForm', 'shapeParts'])

/* 关联形状是否在 formulas 表配置了体积公式（shape_id 有值），决定体积入口是否显示 */
const shapeVolumeReady = computed(() =>
  (props.shapeParts || []).some((shape: any) =>
    (shape.formulas || []).some((item: any) => item.columnar && ['min', 'max'].includes(item.extension?.variant))
  )
)

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  submit: false,
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
  formulas: '',
})

// 新增类型定义
interface FormulaItem {
  name: string
  unit: string
  columnar: string
  condition: {
    shapesIds: any | null
    categoriesIds: number[] | string[] | null
  }
}
const fixedFormula = ref<Record<string, FormulaItem>>({
  volume: {
    name: '体积',
    unit: 'mm^3',
    columnar: '',
    condition: {
      shapesIds: null,
      categoriesIds: null,
    },
  },
  minBreakingTorque: {
    name: '最小破坏扭矩',
    unit: 'N*m',
    columnar: 'π/16*(d-1.0825*P+{threadDeviation}-{Td1})^3*σbMin*X*0.97/1000',
    condition: {
      shapesIds: [2, 12],
      categoriesIds: ['1|2|3|89|90', '!(34|36)'],
    },
  },
  assemblyTorque: {
    name: '装配扭矩',
    unit: 'N*m',
    columnar: '{K}*(Rp*0.785*(d-0.6495*P)^2)*d/1000',
    condition: {
      shapesIds: [2, 12],
      categoriesIds: ['1|2|3|89|90', '!(34|36)'],
    },
  },
  shearLoad: {
    name: '抗剪力',
    unit: 'KN',
    columnar: '0.4553*σbMin*(d-0.6495*P)^2/1000',
    condition: {
      shapesIds: [2, 12],
      categoriesIds: ['1|2|3|89|90', '!(34|36)'],
    },
  },
  tensileLoad: {
    name: '抗拉力',
    unit: 'KN',
    columnar: '0.785*σbMin*(d-0.6495*P)^2/1000',
    condition: {
      shapesIds: [2, 12],
      categoriesIds: ['1|2|3|89|90', '!(34|36)'],
    },
  },
  // antiFatigueLoad: {
  //   name: "抗疲劳",
  //   unit: "mm",
  //   columnar: "",
  //   condition: {
  //     shapesIds: [2, 12],
  //     categoriesIds: null
  //   }
  // },
  // nutTensileLoad: {
  //   name: "螺母抗拉力",
  //   unit: "mm",
  //   columnar: "",
  //   condition: {
  //     shapesIds: [2, 12],
  //     categoriesIds: null
  //   }
  // },
  safeLoad: {
    name: '螺母保证载荷（安全载荷）',
    unit: 'KN',
    columnar: '',
    condition: {
      shapesIds: [1, 12],
      categoriesIds: ['160|162|165|39|40|41|43|52'],
    },
  },
  hardnessToStrength: {
    name: '硬度⇄强度',
    unit: 'MPa',
    columnar: '',
    condition: {
      shapesIds: null,
      categoriesIds: null,
    },
  },
  tolerance: {
    name: '公差计算',
    unit: 'mm',
    columnar: '',
    condition: {
      shapesIds: null,
      categoriesIds: null,
    },
  },
  threadTolerance: {
    name: '螺纹公差',
    unit: 'mm',
    columnar: '',
    condition: {
      shapesIds: ['1|2', 12],
      categoriesIds: null,
    },
  },
  rodWithoutThread: {
    name: '螺纹毛坯',
    unit: 'mm',
    columnar: '',
    condition: {
      shapesIds: [2, 12],
      categoriesIds: null,
    },
  },
  // selfTappingAperture: {
  //   name: "自攻螺纹装配孔径",
  //   unit: "mm",
  //   columnar: "",
  //   condition: {
  //     shapesIds: [2, 12],
  //     categoriesIds: null
  //   }
  // },
  // selfTappingTorque: {
  //   name: "自攻螺纹装配力矩",
  //   unit: "mm",
  //   columnar: "",
  //   condition: {
  //     shapesIds: [2, 12],
  //     categoriesIds: null
  //   }
  // },
  // selfTappingDrawingLoad: {
  //   name: "自攻螺纹抗拉拔力",
  //   unit: "mm",
  //   columnar: "",
  //   condition: {
  //     shapesIds: [2, 12],
  //     categoriesIds: null
  //   }
  // }
})

const formulasActive = ref<any>([])

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

const conditionShow = (item: any) => {
  return (
    conditionValidate(props.productRow.shapes_id, item?.condition?.shapesIds) &&
    conditionValidate(props.productRow.categories_id, item?.condition?.categoriesIds)
  )
}

const handelFormulas = (code: string) => {
  state.formulas = state.formulas === code ? '' : code
  if (!!state.formulas && formulasRef[state.formulas]) {
    // console.log(state.formulas, formulasRef[state.formulas])
    formulasRef[state.formulas].handelDefault()
  }
  nextTick(() => {
    formulasShowRefs.value.scrollIntoView({ behavior: 'smooth', block: 'center' })
  })
}

const handelCleanFormulas = () => {
  state.formulas = ''
}
/**
 * 直径变化处理
 * 根据条件切换显示不同的公式：
 * - 当满足drawingLimit条件时显示第二个公式
 * - 否则显示第一个公式
 */
const diameterChange = () => {
  const { productRow, queryForm } = props
  if (productRow?.drawingLimit?.[queryForm.mon]?.['IMG']) {
    formulasActive.value = evaluateConditionDefault(productRow.drawingLimit?.[queryForm.mon]?.['IMG'], productRow.parameters, queryForm)
      ? productRow.formulas?.[0] || []
      : productRow.formulas?.[1] || []
  } else {
    formulasActive.value = productRow.formulas?.[0] || []
  }
  nextTick(() => {
    if (!!state.formulas && formulasRef[state.formulas]) {
      formulasRef[state.formulas].handelDefault()
    }
  })
}

/**
 * 显示SVG入口方法
 * 组合执行直径判断和SVG加载流程
 */
const handelDefault = () => {
  state.formulas = ''
  diameterChange()
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })

watch(
  props.queryForm,
  () => {
    diameterChange()
    // result.value.thousandWeight = 0;
    // formulasForm.value = { ...state.defaultFormulasForm };
  },
  { deep: true, immediate: false } // 深度监听，非立即触发
)

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style lang="scss" scoped>
.vab-card .el-card__header {
  height: 40px;
  padding: 10px;
  font-size: 16px;
  background: rgba(5, 41, 101, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}
</style>
