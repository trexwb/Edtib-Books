<!--
 * @Author: ${git_name}
 * @Date: 2025-04-14 13:39:04
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-06-24 11:04:52
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsSizeFilter.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="under-tabs-select-box" style="padding-left: 10px">
    <el-row :gutter="20">
      <el-col :lg="18" :md="18" :sm="18" :xl="18" :xs="24">
        <el-form class="custom-el-form" :inline="true" label-width="80px">
          <el-form-item style="width: 200px; padding-top: 10px; margin-bottom: 10px">
            <template #label>
              <div class="under-tabs-img-box">
                <div>直径</div>
                <img alt="直径" :src="diameterImg" />
              </div>
            </template>
            <el-select v-model="queryForm.mon" clearable filterable placeholder="直径" placement="top" @change="handleDiameterChange">
              <el-option
                v-for="item in dropDownDataSet.diameterArr"
                :key="`diameter[${item.value}]`"
                :label="item.label"
                :value="item.value"
              />
            </el-select>
          </el-form-item>
          <el-form-item v-show="dropDownDataSet.lengthArr.length > 0" style="width: 200px; padding-top: 10px; margin-bottom: 10px">
            <template #label>
              <div class="under-tabs-img-box">
                <div>长度</div>
                <img alt="长度" :src="lengthImg" />
              </div>
            </template>
            <el-select
              ref="selectRef"
              v-model="queryForm.diameterLength"
              clearable
              placeholder="长度"
              placement="top"
              @change="handleLengthChange"
              @visible-change="handleVisibleChange"
            >
              <template #header>
                <el-input
                  ref="searchInput"
                  v-model="searchLen"
                  clearable
                  :formatter="(value: string) => value.replace(/[^\d.]/g, '')"
                  :parser="(value: string) => value.replace(/[^\d.]/g, '')"
                  placeholder="可输入数据筛选"
                  size="small"
                  style="width: 165px; height: 30px"
                  @keydown.enter="handleEnter"
                />
              </template>
              <el-option v-if="!!Number(searchLen)" :label="searchLen" :value="searchLen" />
              <el-option v-for="item in filteredOptions" :key="item.value" :label="item.label" :value="item.value" />
            </el-select>
          </el-form-item>
          <el-form-item v-show="dropDownDataSet.pitchArr.length > 0" style="width: 200px; padding-top: 10px; margin-bottom: 10px">
            <template #label>
              <div class="under-tabs-img-box">
                <div>螺距</div>
                <img alt="螺距" :src="pitchImg" />
              </div>
            </template>
            <el-select v-model="queryForm.pitch" clearable filterable placeholder="螺距" placement="top" @change="handlePitchChange">
              <el-option v-for="item in dropDownDataSet.pitchArr" :key="item.value" :label="item.label" :value="item.value" />
            </el-select>
          </el-form-item>
        </el-form>
      </el-col>
      <el-col :lg="2" :md="2" :offset="2" :sm="2" :xl="2" :xs="4">
        <el-tooltip effect="light" :offset="0" placement="top">
          <template #content>
            免责申明：本网站所有资源仅供参考学习，
            <br />
            禁止未经授权的商业用途，
            <br />
            如因依赖从本网站获得的信息并用于商业用途造成的损失，
            <br />
            我们对此不负任何责任。
          </template>
          <div class="notice-title">
            <el-icon color="#fff" size="18">
              <question-filled />
            </el-icon>
            <div class="label-title">商业用途</div>
          </div>
        </el-tooltip>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { getMathEvaluate, formatNumber } from '/@/utils/evaluateCondition'
import { QuestionFilled } from '@element-plus/icons-vue'
import diameterImg from '/@/assets/diameter.png'
import lengthImg from '/@/assets/length.png'
import pitchImg from '/@/assets/pitch.png'
import { ElSelect } from 'element-plus'

const emit = defineEmits(['handelChange', 'updateLength', 'updateQueryForm'])
const props = defineProps(['parameters', 'diameterLength', 'tolerance', 'queryForm'])

// 下拉数据集合
const dropDownDataSet = reactive<{
  diameterArr: Array<{ value: any; label: any }>
  lengthArr: Array<{ value: any; label: any }>
  pitchArr: Array<{ value: any; label: any }>
}>({
  diameterArr: [],
  lengthArr: [],
  pitchArr: [],
})

const searchInput = ref<HTMLInputElement | null>(null)
const selectRef = ref<InstanceType<typeof ElSelect> | null>(null)
const searchLen = ref('')
const filteredOptions = computed(() => {
  if (!searchLen.value) {
    return dropDownDataSet.lengthArr.filter((item: any) => Number(item.value) !== 0)
  }
  return dropDownDataSet.lengthArr.filter((item: any) => Number(item.label) == Number(searchLen.value))
})

const handleEnter = () => {
  if (searchLen.value) {
    props.queryForm.diameterLength = Number(searchLen.value)
    selectRef.value?.blur()
    defaultPitch()
    // emit('updateLength', searchLen.value);
    // emit('handelChange')
  }
}

const handleVisibleChange = (isVisible: any) => {
  if (!isVisible) {
    searchLen.value = ''
  } else {
    setTimeout(() => {
      if (searchInput.value) {
        searchInput.value.focus()
      }
    }, 0)
  }
}

// 直径改变
const handleDiameterChange = (value: any) => {
  props.queryForm.mon = value
  defaultDiameterLength()
  defaultPitch()
}
// 长度改变
const handleLengthChange = (value: any) => {
  props.queryForm.diameterLength = Number(value)
  defaultPitch()
}
// 螺距改变
const handlePitchChange = (value: any) => {
  props.queryForm.pitch = Number(value)
  emit('handelChange')
}

// 自定义排序函数
function getPriority(s: string): number {
  if (!s.length) return 2 // 空字符串视为其他符号
  const firstChar = s[0]
  if (/[a-zA-Z]/.test(firstChar)) return 0 // 字母
  if (/\d/.test(firstChar)) return 2 // 数字
  return 1 // 其他符号
}
function customSort(arr: string[]): string[] {
  return [...arr].sort((a, b) => {
    const priorityA = getPriority(a)
    const priorityB = getPriority(b)

    // 1. 按优先级排序（字母 > 数字 > 其他）
    if (priorityA !== priorityB) {
      return priorityA - priorityB
    }

    // 2. 同优先级时，进一步处理字母+数字的情况
    if (priorityA === 0) {
      // 字母开头
      // 提取字母部分和数字部分（如 "M10" → ["M", "10"]）
      const matchA = a.match(/^([a-zA-Z@#$%^&]*)(\d*)/)
      const matchB = b.match(/^([a-zA-Z@#$%^&]*)(\d*)/)

      const lettersA = matchA?.[1] || ''
      const lettersB = matchB?.[1] || ''
      const numA = matchA?.[2] ? parseInt(matchA[2], 10) : NaN
      const numB = matchB?.[2] ? parseInt(matchB[2], 10) : NaN

      // 2.1 先比较字母部分
      if (lettersA !== lettersB) {
        return lettersA.localeCompare(lettersB)
      }

      // 2.2 字母相同，比较数字部分
      if (!isNaN(numA) && !isNaN(numB)) {
        return numA - numB // 按数值排序
      }

      // 2.3 无法提取数字，按字典序排序
      return a.localeCompare(b)
    } else if (priorityA === 2) {
      // 数字
      // 2.2 字母相同，比较数字部分
      if (Number(a) && Number(b)) {
        return Number(a) - Number(b) // 按数值排序
      }
    }

    // 3. 其他情况（数字开头或其他符号开头），按字典序排序
    return a.localeCompare(b)
  })
}
function isStringNumeric(str: string) {
  return !isNaN(Number(str)) && !isNaN(parseFloat(str))
}

const defaultDiameterLength = () => {
  dropDownDataSet.lengthArr = []
  if (props.diameterLength && props.diameterLength[props.queryForm.mon]) {
    // 将数组中的元素统一转换为数字类型
    const diameterLengthArr = props.diameterLength[props.queryForm.mon].map(String)
    // 对数组进行升序排序
    diameterLengthArr.sort((a: string, b: string) => Number(a) - Number(b))
    props.queryForm.diameterLength =
      (isStringNumeric(diameterLengthArr[0])
        ? diameterLengthArr[0]
        : formatNumber(getMathEvaluate(diameterLengthArr[0], props.queryForm.variables))) || 0
    diameterLengthArr.forEach((key: any) => {
      const labelKey = Object.keys(props.queryForm.variables).reduce((acc, key) => {
        return acc.replace(new RegExp(key, 'g'), props.queryForm.variables[key])
      }, key)
      dropDownDataSet.lengthArr.push({
        value: isStringNumeric(key) ? key : formatNumber(getMathEvaluate(key, props.queryForm.variables)),
        label: `${labelKey}`, // key
      })
    })
  }
}

const defaultPitch = () => {
  dropDownDataSet.pitchArr = []
  if (props.parameters[props.queryForm.mon]) {
    props.queryForm.diameter = props.parameters[props.queryForm.mon]['d'] || 0
    if (props.parameters[props.queryForm.mon] && props.parameters[props.queryForm.mon]['P']) {
      const pitchValue = (props.parameters[props.queryForm.mon]['P'] || '').replace('\/', '|')
      const pitchLabel = (props.parameters[props.queryForm.mon]['Pitch'] || pitchValue).replace('\/', '|')
      if (pitchValue.includes('|')) {
        const pitchValArr = pitchValue.split('|')
        const pitchLabArr = pitchLabel.split('|')
        for (let i = 0; i < pitchValArr.length; i++) {
          dropDownDataSet.pitchArr.push({
            value: Number(pitchValArr[i] || 0),
            label: Number(pitchLabArr[i] || 0),
          })
        }
        props.queryForm.pitch = Number(pitchValue.split('|')[0] || 0)
      } else {
        dropDownDataSet.pitchArr.push({
          value: Number(pitchValue || 0),
          label: Number(pitchLabel || 0),
        })
        props.queryForm.pitch = Number(pitchValue || 0)
      }
    }
  }
  emit('handelChange')
}

const handelDefault = () => {
  dropDownDataSet.diameterArr = []
  props.queryForm.mon = ''
  props.queryForm.diameter = 0
  props.queryForm.diameterLength = 0
  props.queryForm.pitch = 0
  if (props.parameters) {
    // 将数组中的元素统一转换为数字类型
    // const diameterArr = customSort(Object.keys(props.parameters))
    const result = {} as any
    const i = {} as any
    for (const [key, value] of Object.entries<{ d?: any }>(props.parameters || {})) {
      if (value.d) {
        i[value.d] = (i[value.d] || 0) + 1
        if (result[value.d]) {
          result[value.d + i[value.d]] = key
        } else {
          result[value.d] = key
        }
      } else {
        result[key] = key
      }
    }
    const keysSort = customSort(Object.keys(result || {}))
    const diameterArr = keysSort.map((item: any) => result[item])

    props.queryForm.mon = diameterArr[0] || ''
    diameterArr.forEach((key) => {
      dropDownDataSet.diameterArr.push({
        value: key,
        label: key,
      })
    })
    defaultDiameterLength()
    defaultPitch()
  }
}

// 暴露组件方法
defineExpose({ handelDefault })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style scoped lang="scss"></style>
