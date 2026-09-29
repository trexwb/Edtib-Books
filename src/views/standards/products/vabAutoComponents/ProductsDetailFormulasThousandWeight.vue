<!--
 * @Author: ${git_name}
 * @Date: 2025-05-08 13:13:58
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-08-06 17:49:43
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailFormulasThousandWeight.vue
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
      <el-form-item label="选择:">
        <el-select v-model="formulasForm.material" placeholder="" style="width: 120px" @change="changeResult">
          <template #prefix>材料</template>
          <el-option v-for="item in state.material" :key="`material[${item.value}]`" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-input v-model="result" disabled style="width: 200px">
          <template #append>千支重(kg)</template>
        </el-input>
      </el-form-item>
      <el-form-item label="X" label-width="5px">
        <el-input
          v-model="formulasForm.money"
          :formatter="(value: string) => value.replace(/[^\d.]/g, '')"
          :parser="(value: string) => value.replace(/[^\d.]/g, '')"
          style="width: 200px"
        >
          <template #append>元/kg</template>
        </el-input>
      </el-form-item>
      <el-form-item label="=" label-width="5px">
        <el-text style="font-size: 28px; font-weight: bold" type="primary">
          {{ formatNumber(Number(result) * Number(formulasForm.money) || 0) }}
          元/千支
        </el-text>
        <el-icon
          color="#052965"
          size="28"
          style="margin-left: 10px; cursor: pointer"
          title="复制"
          @click="handleCopy(result * formulasForm.money, currentFormula.name)"
        >
          <document-copy />
        </el-icon>
      </el-form-item>
    </el-form>
  </vab-card>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { DocumentCopy, Close } from '@element-plus/icons-vue'
import { formatNumber, evaluateFormula } from '/@/utils/evaluateCondition'
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

const result = ref(0)

const formulasForm = ref<any>(JSON.parse(JSON.stringify(state.defaultFormulasForm)))
// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}
// 计算全部公式
const changeResult = () => {
  if (state.submit) return
  state.submit = true
  result.value = 0
  // if (!formulasForm.value.material) return;
  try {
    // 防御性拷贝原始variables
    const originalVariables = props.queryForm?.variables || {}
    const variables = { ...originalVariables }
    variables['ρ'] = formulasForm.value.material || 0

    // 计算固定公式
    result.value = evaluateFormula(props.currentFormula?.columnar, props.formulas, variables)
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
  state.material = []
  Object.entries(material.value || {}).forEach(([key, value]) => {
    state.material.push({
      value: key,
      label: value as string,
    })
  })
  changeResult()
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })
</script>

<style lang="scss" scoped></style>
