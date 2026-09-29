<!--
 * @Author: 编程专家
 * @Date: 2026-09-08
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasVolume.vue
 * @Description: 体积计算卡片：按产品关联形状在 formulas 表中的体积公式（shape_id 关联、type=1）代入当前公称尺寸参数求和
 * Copyright (c) 2026 by 杭州大美, All Rights Reserved.
-->
<template>
  <vab-card>
    <template #header>
      <div class="card-header">
        <div>体积（关联形状求和 · 最小 / 最大）</div>
        <el-icon @click="closeFormula">
          <close />
        </el-icon>
      </div>
    </template>
    <el-table v-loading="state.loading" :border="true" :data="state.parts" size="small" :stripe="true">
      <el-table-column label="形状" min-width="120">
        <template #default="{ row }">
          <span>{{ row.name }}</span>
          <el-tag v-if="row.code" class="volume-code" size="small" type="info">{{ row.code }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column align="right" label="体积最小 (mm³)" width="130">
        <template #default="{ row }">
          {{ formatNumber(row.volumeMin, 3) }}
        </template>
      </el-table-column>
      <el-table-column align="right" label="体积最大 (mm³)" width="130">
        <template #default="{ row }">
          {{ formatNumber(row.volumeMax, 3) }}
        </template>
      </el-table-column>
    </el-table>
    <div class="volume-total">
      <span>体积最小</span>
      <b>{{ formatNumber(state.totalMin, 3) }}</b>
      <span class="volume-unit">mm³</span>
      <el-icon class="volume-copy" color="#052965" size="18" title="复制体积最小" @click="handleCopy('体积最小', state.totalMin)">
        <document-copy />
      </el-icon>
    </div>
    <div class="volume-total">
      <span>体积最大</span>
      <b>{{ formatNumber(state.totalMax, 3) }}</b>
      <span class="volume-unit">mm³</span>
      <el-icon class="volume-copy" color="#052965" size="18" title="复制体积最大" @click="handleCopy('体积最大', state.totalMax)">
        <document-copy />
      </el-icon>
    </div>
    <div v-if="!state.hasMon" class="volume-tip">请先选择公称尺寸后再计算体积</div>
    <div v-else-if="state.zeroTotal" class="volume-tip">计算结果为 0，请检查所选规格是否缺少长度等参数</div>
  </vab-card>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { DocumentCopy, Close } from '@element-plus/icons-vue'
import { evaluateFormula, formatNumber } from '/@/utils/evaluateCondition'
import clip from '/@/utils/clipboard'

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { defaultItem } = storeToRefs(aclStore)

const templateName = 'StandardsProductsDetailFormulasVolume'
defineOptions({
  name: templateName,
})

/* 事件和全局方法 */
const emit = defineEmits(['close'])
// 定义 props 类型
const props = defineProps(['currentFormula', 'productRow', 'queryForm', 'formulas', 'shapeParts'])

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  parts: [] as any[], // 参与求和的形状分项
  totalMin: 0, // 体积最小合计 mm³
  totalMax: 0, // 体积最大合计 mm³
  hasMon: true, // 是否已选择公称尺寸
  zeroTotal: false, // 总体积为 0（参数缺失提示）
  defaultLang: defaultItem.value.languages || 'zh-cn',
})

/**
 * 计算体积：取关联形状在 formulas 表中的体积公式（shape_id 关联、type=1），代入当前公称尺寸变量求值后求和
 * 未配置体积公式的形状不参与求和（静默降级）
 */
const compute = () => {
  state.hasMon = !!props.queryForm?.mon
  const variables = props.queryForm?.variables || {}
  const parts: any[] = []
  let totalMin = 0
  let totalMax = 0
  ;(props.shapeParts || []).forEach((shape: any) => {
    // 体积公式双变体：extension.variant = min | max（后台形状维护时标记）
    const minFormula = (shape.formulas || []).find((item: any) => item.extension?.variant === 'min' && item.columnar)
    const maxFormula = (shape.formulas || []).find((item: any) => item.extension?.variant === 'max' && item.columnar)
    if (!minFormula && !maxFormula) return
    const name = shape.names?.[state.defaultLang] || Object.values(shape.names || {})[0] || `形状#${shape.id}`
    const volumeMin = minFormula ? Number(evaluateFormula(minFormula.columnar, props.formulas || {}, variables)) || 0 : 0
    const volumeMax = maxFormula ? Number(evaluateFormula(maxFormula.columnar, props.formulas || {}, variables)) || 0 : 0
    parts.push({ name, columnar: (minFormula || maxFormula).columnar, volumeMin, volumeMax })
    totalMin += volumeMin
    totalMax += volumeMax
  })
  state.parts = parts
  state.totalMin = totalMin
  state.totalMax = totalMax
  state.zeroTotal = parts.length > 0 && totalMin === 0 && totalMax === 0
}

// 复制
const handleCopy = (label: string, value: number) => {
  clip(formatNumber(value, 3), label)
}

const closeFormula = () => {
  emit('close')
}

// 暴露组件方法给父组件
const handelDefault = () => {
  compute()
}

defineExpose({ handelDefault })

watch(
  () => props.queryForm,
  () => compute(),
  { deep: true }
)
</script>

<style lang="scss" scoped>
.volume-code {
  margin-left: 6px;
}

.volume-total {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: flex-end;
  padding: 12px 4px 0;
  font-size: 15px;

  b {
    font-size: 22px;
    color: var(--el-color-primary);
  }

  .volume-unit {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }

  .volume-copy {
    cursor: pointer;
  }
}

.volume-tip {
  padding-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  text-align: right;
}
</style>
