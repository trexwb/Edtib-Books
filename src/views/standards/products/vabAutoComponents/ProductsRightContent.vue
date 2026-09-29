<!--
 * @Author: ${git_name}
 * @Date: 2025-04-16 12:16:17
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-22 15:03:31
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsRightContent.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="right-content">
    <div>
      <!-- <el-scrollbar style="max-height: 430px; height: 100%"> -->
      <ul class="right-content-list">
        <li>
          <div class="label">MON</div>
          <div class="value">{{ queryForm.mon }}</div>
        </li>
        <li v-for="(item, index) in showParameters" v-show="variables[index] && !!item" :key="`parameters[${index}]`">
          <div class="label">{{ variables[index]?.code }}</div>
          <div class="value">{{ item }}</div>
        </li>
      </ul>
      <!-- </el-scrollbar> -->
      <ul>
        <li v-if="extension.unit">
          <el-select v-model="state.unit" placeholder="" style="width: 100%" @change="dataConversion">
            <template #prefix>单位</template>
            <el-option v-for="item in ['inch', 'mm']" :key="`unit[${item}]`" :label="item" :value="item" />
          </el-select>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { formatNumber } from '/@/utils/evaluateCondition'

const emit = defineEmits(['updateUnit'])
const props = defineProps(['parameters', 'variables', 'extension', 'queryForm'])
const showParameters = ref<any>({})
const state = reactive({
  unit: props.extension?.unit || 'mm',
})

const dataConversion = () => {
  const lengthFactor = 25.4
  if (state.unit == 'inch') {
    Object.keys(showParameters.value).map((key: string) => {
      if (key !== 'd' && key !== 'P' && key !== 'Pitch' && key !== 'pitch' && !Number.isNaN(Number(showParameters.value[key]))) {
        showParameters.value[key] = formatNumber(showParameters.value[key] / lengthFactor)
      }
    })
  } else if (state.unit == 'mm') {
    Object.keys(showParameters.value).map((key: string) => {
      if (key !== 'd' && key !== 'P' && key !== 'Pitch' && key !== 'pitch' && !Number.isNaN(Number(showParameters.value[key]))) {
        showParameters.value[key] = formatNumber(showParameters.value[key] * lengthFactor)
      }
    })
  }
  // 触发svg图单位换算
  emit('updateUnit', state.unit)
}

/**
 * 监听props变化
 * 当queryForm变化时自动更新SVG显示
 */
watch(
  props,
  () => {
    state.unit = props.extension?.unit || 'mm'
    showParameters.value = {}
    if (props.parameters[props.queryForm.mon]) {
      Object.keys(props.parameters[props.queryForm.mon]).map((key: string) => {
        if (
          key !== 'd' &&
          key !== 'P' &&
          key !== 'Pitch' &&
          key !== 'pitch' &&
          (typeof props.parameters[props.queryForm.mon][key] == 'string' || typeof props.parameters[props.queryForm.mon][key] == 'number')
        ) {
          showParameters.value[key] = props.parameters[props.queryForm.mon][key].toString()
        } else if (key == 'P' || key == 'Pitch' || key == 'pitch') {
          showParameters.value['P'] = (
            props.parameters[props.queryForm.mon]['Pitch'] ||
            props.parameters[props.queryForm.mon]['pitch'] ||
            props.parameters[props.queryForm.mon]['P']
          ).toString()
        }
      })
    }
  },
  { immediate: true, deep: true }
)

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {})
</script>

<style lang="scss">
.right-content {
  position: absolute;
  top: 1px;
  right: 1px;
  z-index: 1000;
  width: 140px;
  height: auto;
  max-height: 100%;
  background: #f2f3f6;

  .right-content-list {
    width: 100%;
    height: auto;
    max-height: 435px;
    overflow: hidden;
    overflow-y: auto;
  }

  .right-content-img {
    padding-top: 20px;
    text-align: center;
    background: #fff;

    img {
      width: 100%;
      height: auto;
    }
  }

  ul,
  li {
    width: 100%;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  li {
    display: flex;
    justify-content: flex-start;
    width: 100%;
    height: 28px;
    line-height: 28px;

    .label,
    .value {
      flex: 1;
      text-align: center;
    }

    .label {
      color: #fff;
      background: var(--default-color);
      border-bottom: 1px solid #fff;
    }

    .value {
      color: var(--default-color);
      border-bottom: 1px solid #ddd;
    }
  }
}
</style>
