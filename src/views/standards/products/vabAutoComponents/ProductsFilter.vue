<!--
 * @Author: ${git_name}
 * @Date: 2025-04-10 09:22:11
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-08 14:50:03
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsFilter.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="products-filter-box">
    <vab-query-form style="width: 100%; padding-top: 10px">
      <vab-query-form-top-panel style="width: 100%">
        <el-form label-width="70px" @submit.prevent>
          <el-form-item label="模糊搜索">
            <el-input
              v-model="keywords"
              clearable
              placeholder="请输入产品标准号或产品名称模糊搜索"
              style="width: 335px"
              @keyup.enter="searchKeywords"
            >
              <template #append>
                <el-button :icon="Search" @click="searchKeywords" />
              </template>
            </el-input>
          </el-form-item>
        </el-form>
      </vab-query-form-top-panel>
      <!-- 标准分类 -->
      <vab-query-form-top-panel style="width: 100%">
        <el-form label-width="70px">
          <el-form-item v-if="standardsRows.length > 0" class="standard-form-box" label="标准分类">
            <div class="standard-tag" :class="[isStandardExpanded ? 'isStandardExpanded' : '', standardStatus ? 'standardCollapsed' : '']">
              <el-radio-group v-model="standardId" @change="changeStandard">
                <el-radio value="">
                  <el-tag :class="[standardId === '' ? 'active' : 'primary']" :color="standardId === '' ? '#052965' : ''" size="small">
                    不限
                  </el-tag>
                </el-radio>
                <el-radio v-for="(data, index) in standardsRows" :key="`standard[${index}]`" :label="data.title" :value="data.id">
                  <el-tag
                    :class="[Number(standardId) === Number(data.id) ? 'active' : 'primary']"
                    :color="Number(standardId) === Number(data.id) ? '#052965' : ''"
                    size="small"
                  >
                    {{ data.names[state.defaultLang] }}
                  </el-tag>
                </el-radio>
              </el-radio-group>
            </div>
            <el-text class="more-standard" @click="toggleExpandStandardId">
              {{ isStandardExpanded ? '收起' : '展开' }}
              <el-icon class="arrow-img" :class="[isStandardExpanded ? 'isRoute' : '']">
                <arrow-down-bold />
              </el-icon>
            </el-text>
          </el-form-item>
        </el-form>
      </vab-query-form-top-panel>
      <!-- 形状 -->
      <vab-query-form-top-panel
        class="shape-form-box"
        :class="[isShapeExpanded ? 'shapeExpanded' : '', shapeStatus ? 'shapeCollapsed' : '']"
        style="width: 100%"
      >
        <el-form label-width="70px">
          <div class="shape-tag-box">
            <div
              v-for="locationIndex in Object.keys(configuration.shape)"
              v-show="!['1', '3', '9'].includes(locationIndex) && (shapeId[locationIndex] || shapesRows[locationIndex]?.length > 0)"
              :key="`shapeLocation[${locationIndex}]`"
              style="margin-bottom: 10px"
            >
              <el-form-item class="shape-flex-form-item" :label="configuration.shape[locationIndex][state.defaultLang]">
                <el-radio-group v-model="shapeId[locationIndex]" @change="(value) => changeShape(value, locationIndex)">
                  <div class="shape-flex-radio-group">
                    <div
                      v-show="shapeId[locationIndex] || shapesRows[locationIndex]?.length > 0"
                      style="height: 28px; padding-right: 15px; line-height: 28px"
                    >
                      <el-radio value="">
                        <el-tag
                          :class="[shapeId[locationIndex] === '' ? 'active' : 'primary']"
                          :color="shapeId[locationIndex] === '' ? '#052965' : ''"
                          size="small"
                        >
                          不限
                        </el-tag>
                      </el-radio>
                    </div>
                    <div style="padding-top: 3px">
                      <template v-for="item in shapesUsable[locationIndex]" :key="`shape[${item.id}]`">
                        <span
                          v-show="
                            (!shapeId[locationIndex] || [shapeId[locationIndex]].includes(item.id)) &&
                            (shapesRows[locationIndex] || []).map((item: any) => item.id).includes(item.id)
                          "
                          style="position: relative; display: inline-block; width: 28px; height: 28px; margin-right: 20px"
                        >
                          <el-radio class="select-radio" :label="item.id" :value="item.id">
                            <el-popover :offset="4" placement="bottom" popper-class="shape-popover-box" :show-after="300" trigger="hover">
                              <template #default>
                                <div class="shape-popover-content">
                                  <el-image
                                    :alt="item.names[state.defaultLang]"
                                    class="shape-popover-image"
                                    :src="`${item.covers[0]?.url}!a200`"
                                  >
                                    <template #error>
                                      <div class="image-slot"></div>
                                    </template>
                                  </el-image>
                                  <div class="shape-popover-title">{{ item.names[state.defaultLang] }}</div>
                                </div>
                              </template>
                              <template #reference>
                                <el-image
                                  :alt="item.names[state.defaultLang]"
                                  :src="`${item.covers[0]?.url}!a50`"
                                  style="width: 28px; height: 28px"
                                >
                                  <template #error>
                                    <div class="image-slot">
                                      <el-icon size="16"><icon-picture /></el-icon>
                                    </div>
                                  </template>
                                </el-image>
                              </template>
                            </el-popover>
                          </el-radio>
                          <el-popover :offset="4" placement="bottom" popper-class="shape-popover-box" :show-after="300" trigger="hover">
                            <template #default>
                              <div class="shape-popover-content">
                                <el-image
                                  :alt="item.names[state.defaultLang]"
                                  class="shape-popover-image"
                                  :src="`${item.covers[0]?.url}!a200`"
                                >
                                  <template #error>
                                    <div class="image-slot"></div>
                                  </template>
                                </el-image>
                                <div class="shape-popover-title">{{ item.names[state.defaultLang] }}</div>
                              </div>
                            </template>
                            <template #reference>
                              <div
                                v-show="Number(item.id) === Number(shapeId[locationIndex])"
                                class="shape-checked"
                                @click="handleShapeClick(item.id, locationIndex)"
                              >
                                <div class="triangle">
                                  <el-icon class="shape-checked-icon"><select /></el-icon>
                                </div>
                              </div>
                            </template>
                          </el-popover>
                        </span>
                      </template>
                    </div>
                  </div>
                </el-radio-group>
              </el-form-item>
            </div>
          </div>
        </el-form>
      </vab-query-form-top-panel>
    </vab-query-form>
    <!-- <div style="height: 15px; width: 100%"></div> -->
    <!-- <el-button v-show="Object.keys(shapesRows).length > 5" @click="toggleExpandShapeId" class="more-shape">
      {{ isShapeExpanded ? '收起更多形状筛选' : '展开更多形状筛选' }}
      <el-icon class="arrow-img" :class="[isShapeExpanded ? 'isRoute' : '']">
        <ArrowDownBold />
      </el-icon>
    </el-button> -->
  </div>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { standardsAll, shapesAll, productsFilterOptions, requestBridge } from '/@/api/products'
import { Search, Select, ArrowDownBold, Picture as IconPicture } from '@element-plus/icons-vue'
// [迁移调整] 本地聚合查询统一走 src/bridge 桥接层（Tauri invoke）
import { bridge } from '/@/bridge'

/* 组件模板名称 */
const templateName = 'StandardsProductsFilter'

/* 定义组件选项 */
defineOptions({
  name: templateName, // 注册组件名称，用于调试和keep-alive缓存标识
})

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { configuration, defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
  standards: [],
  shapes: {},
  defaultShapesIds: [],
})

const props = defineProps(['categorieResult'])
const emit = defineEmits(['handle-change', 'handle-search']) // 定义组件事件

const keywords = ref<string>('')
const standardId = ref<number | string>('')
const standardResult = ref<any>(null)
const standardsRows = ref<any[]>([])
// 标准分类是否展开
const isStandardExpanded = ref(false)
// 标准分类初始化时不触发动画,操作时触发动画
const standardStatus = ref(false)
// 标准分类切换展开/收起状态
const toggleExpandStandardId = () => {
  isStandardExpanded.value = !isStandardExpanded.value
  if (isStandardExpanded.value) {
    standardStatus.value = false
  } else {
    standardStatus.value = true
  }
}

// 形状选中
const shapeId = reactive<Record<string, string>>({})
const shapeResult = ref<any[]>([])
// 形状是否展开
const isShapeExpanded = ref(true)
// 形状初始化时不触发动画,操作时触发动画
const shapeStatus = ref(false)
// 形状切换展开/收起状态
const toggleExpandShapeId = () => {
  isShapeExpanded.value = !isShapeExpanded.value
  if (isShapeExpanded.value) {
    shapeStatus.value = false
  } else {
    shapeStatus.value = true
  }
}
// 形状列表
const shapesRows = ref<any>({})
const shapesUsable = ref<any>({})
const shapeTransform = (intersection: any, clearResult: boolean) => {
  // 重组形状数据：按形状类型分组，过滤出有效形状
  const shapesSet = new Set(intersection) // 转换为 Set 提高查找效率
  shapesRows.value = Object.keys(state.shapes).reduce((acc: any, key) => {
    // 过滤当前键对应的数组
    const filteredArray = state.shapes[key].filter((row: any) => shapesSet.has(row.id))
    // 如果过滤后的数组不为空，则保留该键
    if (filteredArray.length > 0) {
      acc[key] = filteredArray
    }
    return acc
  }, {})
  // 自动展开判断：当有效形状类型少于4种时展开面板
  if (Object.keys(shapesRows.value).length < 4) isShapeExpanded.value = true
  // 删除已经选中的不存在的形状,清理无效选中项（已移除的形状选择）
  if (shapeResult.value && clearResult) {
    // 提取 data 中的所有 id
    const dataIds = new Set(
      Object.values(shapesRows.value)
        .flat()
        .map((item: any) => item.id)
    )
    // 过滤 have 数组，保留 id 存在于 dataIds 中的对象
    shapeResult.value = (shapeResult.value || []).filter((item) => dataIds.has(item.id))
    // shapeId[key] = (shapeResult.value || []).map(item => item.id);
    Object.keys(shapeId).forEach((key) => {
      if (!dataIds.has(Number(shapeId[key]))) {
        shapeId[key] = ''
      }
    })
  }
}
/**
 * 本地聚合 products_filter（IPC/Electron 模式）
 * 与后端 getFilterOptionShapes 语义一致：在已选 分类/标准/形状 组合下，
 * 求满足条件的正式启用产品（products.status=1 且未软删）覆盖的全部形状ID。
 * 注意：筛选条件全部作用于"候选产品集合"，形状聚合取候选产品全部 shape 行，
 * 不得按行过滤 category_id（category 行无 shape_id、shape 行无 category_id）。
 */
const localResolveShapeIds = async (filter: any): Promise<number[]> => {
  // [迁移调整] 统一走 src/bridge 桥接层（Tauri invoke）
  const api = bridge
  if (!api?.db?.findAll) throw new Error('IPC db unavailable')
  const toArr = (res: any): any[] => (Array.isArray(res) ? res : Array.isArray(res?.data) ? res.data : [])
  const [fRes, pRes] = await Promise.all([
    api.db.findAll('products_filter', { product_type: 0 }),
    api.db.findAll('products', { status: 1 }),
  ])
  const fRows: any[] = toArr(fRes)
  const pRows: any[] = toArr(pRes)
  // 候选产品：启用 + 未软删（IPC findAll 主进程可能不过滤软删，此处兜底），标准匹配
  let productIds = new Set(pRows.filter((r: any) => Number(r.status) === 1 && !r.deleted_at).map((r: any) => Number(r.id)))
  if (filter.standard_id !== undefined && filter.standard_id !== null && filter.standard_id !== '') {
    const stdPids = new Set(
      pRows
        .filter((r: any) => productIds.has(Number(r.id)) && Number(r.standard_id ?? r.standardId) === Number(filter.standard_id))
        .map((r: any) => Number(r.id))
    )
    productIds = stdPids
  }
  // 分类：与后端一致取最后一级；约束作用于候选产品（产品须有该分类的关联行）
  if (filter.category_id !== undefined && filter.category_id !== null && filter.category_id !== '') {
    const catId = Number(Array.isArray(filter.category_id) ? filter.category_id[filter.category_id.length - 1] : filter.category_id)
    const catPids = new Set(fRows.filter((r: any) => Number(r.category_id) === catId).map((r: any) => Number(r.product_id)))
    productIds = new Set([...productIds].filter((pid: number) => catPids.has(pid)))
  }
  // 已选形状：每个形状要求产品均关联（AND 语义），同样只收窄产品集合
  const selIds = (filter.shape_ids || []).map((v: any) => Number(v)).filter((n: number) => n > 0)
  for (const sid of selIds) {
    const sidPids = new Set(fRows.filter((r: any) => Number(r.shape_id) === sid).map((r: any) => Number(r.product_id)))
    productIds = new Set([...productIds].filter((pid: number) => sidPids.has(pid)))
  }
  // 聚合：候选产品范围内的全部 shape 行（含与分类无直接行级绑定、仅经产品闭包广播的形状）
  return [
    ...new Set(
      fRows.filter((r: any) => productIds.has(Number(r.product_id)) && Number(r.shape_id) > 0).map((r: any) => Number(r.shape_id))
    ),
  ]
}

/**
 * 请求筛选联动可用形状（由 products_filter 实时派生，替代旧版 standards/shapes.extension.shapes）
 * @returns 形状ID数组；返回 null 表示数据源不可用或请求已过期（调用方保持现有候选不变）
 */
let filterSeq = 0
const fetchAvailableShapes = async (): Promise<number[] | null> => {
  const mySeq = ++filterSeq
  const filter: any = {}
  if (standardId.value !== '' && standardId.value !== null && standardId.value !== undefined) {
    filter.standard_id = Number(standardId.value)
  }
  if (props.categorieResult?.id) {
    filter.category_id = Number(props.categorieResult.id)
  }
  const selected = (shapeResult.value || []).map((item: any) => Number(item.id)).filter((n: number) => n > 0)
  if (selected.length > 0) filter.shape_ids = selected
  // 无条件时保持全量形状候选，无需请求
  if (!filter.standard_id && !filter.category_id && (!filter.shape_ids || filter.shape_ids.length === 0)) {
    return state.defaultShapesIds || []
  }
  try {
    let shapeIds: number[] = []
    let resolvedByIpc = false
    if (requestBridge.isElectron()) {
      try {
        // [迁移调整] 本地 IPC 不可用（Tauri 侧命令尚未实现/调用异常）时继续走 HTTP，
        // 保证桥接占位阶段与旧 Electron 版一致的功能可用性
        shapeIds = await localResolveShapeIds(filter)
        resolvedByIpc = true
      } catch (ipcError) {
        console.warn('[ProductsFilter] 本地 IPC 聚合失败，降级 HTTP', ipcError)
      }
    }
    if (!resolvedByIpc) {
      const { data } = await productsFilterOptions({ filter })
      shapeIds = (data?.shape_ids || []) as number[]
    }
    if (mySeq !== filterSeq) return null
    return shapeIds
  } catch {
    if (mySeq !== filterSeq) return null
    return null
  }
}

/**
 * 形状候选收敛：按当前 分类/标准/已选形状 刷新可用形状候选
 * @param clearResult true 时清理已失效选中（分类/标准切换场景）
 */
const refreshShapes = async (clearResult: boolean) => {
  const ids = await fetchAvailableShapes()
  if (ids === null) return
  nextTick(() => {
    shapeTransform(ids, clearResult)
  })
}

// 标准分类选中事件
const changeStandard = async (value: any) => {
  keywords.value = ''
  standardResult.value = standardsRows.value.find((item: any) => Number(item.id) === Number(value)) || null
  await refreshShapes(true)
  emit('handle-change', keywords.value, standardResult.value, shapeResult.value)
}

/**
 * 形状候选刷新：形状切换后保留已选、仅收敛其它候选
 */
const shapeChangeShape = async () => {
  await refreshShapes(false)
}
// 形状选中事件
const changeShape = async (value: any, key: string) => {
  const currentKey = Number(key)
  if (isNaN(currentKey)) return

  const currentValue = Number(value)
  const results = [...(shapeResult.value ?? [])]
  const shapes = shapesRows.value?.[key] ?? []

  if (value === '') {
    shapeResult.value = results.filter((item) => item && Number(item.key) !== currentKey)
  } else {
    if (isNaN(currentValue)) return
    const isValidRow = shapes.some((item: any) => Number(item.id) === currentValue)
    if (!isValidRow) return

    const keyIndex = results.findIndex((item) => Number(item.key) === currentKey)
    if (keyIndex >= 0) results.splice(keyIndex, 1)

    const exists = results.some((item) => Number(item.id) === currentValue)
    if (!exists) {
      const targetRow = shapes.find((item: any) => Number(item.id) === currentValue)
      if (targetRow) {
        results.push({
          id: currentValue,
          names: targetRow.names,
          key: String(currentKey),
          label: configuration.value.shape[key][state.defaultLang],
        })
      }
    }
    shapeResult.value = results
  }
  keywords.value = ''
  await shapeChangeShape()
  emit('handle-change', keywords.value, standardResult.value, shapeResult.value)
}

// 形状单选， 再次点击取消选中
const handleShapeClick = async (value: any, key: any) => {
  // 使用类型明确的参数
  const currentValue = shapeId[key]
  shapeId[key] = currentValue === value ? '' : value
  // 使用findIndex提前定位目标元素
  const targetIndex = (shapeResult.value || []).findIndex(
    (item) => String(item.id) === String(value) // 统一转为字符串比较
  )
  if (targetIndex > -1) {
    shapeResult.value.splice(targetIndex, 1) // 使用splice确保响应式更新
  }
  keywords.value = ''
  await shapeChangeShape()
  emit('handle-change', keywords.value, standardResult.value, shapeResult.value)
}

// 搜索
const searchKeywords = () => {
  // if (!keywords) return;
  standardsRows.value = JSON.parse(JSON.stringify(state.standards))
  shapesRows.value = JSON.parse(JSON.stringify(state.shapes))
  standardId.value = ''
  for (const key in shapeId) {
    shapeId[key] = ''
  }
  emit('handle-change', keywords.value, null, null)
}
// 获取标准分类数据
const fetchStandards = async () => {
  const { data } = await standardsAll()
  state.standards = data.list || []
  standardsRows.value = JSON.parse(JSON.stringify(state.standards))
}
// 获取形状分类数据
const fetchShapes = async () => {
  const { data } = await shapesAll()
  // 后端 HTTP 返回已按 location 分组对象 { location: [...] }；IPC 直连返回扁平数组，需统一为分组对象
  if (Array.isArray(data)) {
    state.shapes = data.reduce((acc: any, item: any) => {
      const key = String(item.location)
      if (!acc[key]) acc[key] = []
      acc[key].push(item)
      return acc
    }, {})
  } else {
    state.shapes = data || {}
  }
  for (const key in state.shapes) {
    if (state.shapes.hasOwnProperty(key)) shapeId[key] = ''
  }
  shapesRows.value = JSON.parse(JSON.stringify(state.shapes))
  shapesUsable.value = JSON.parse(JSON.stringify(state.shapes))
  state.defaultShapesIds = Object.values(state.shapes)
    .flat()
    .map((item: any) => item.id) // 提取每个对象的 id
}

const handleReset = (standardRest: any, shapeRest: any) => {
  // console.log('shapeRest:', shapeRest)
  keywords.value = ''
  standardResult.value = standardRest || null
  standardId.value = standardRest?.id || ''
  shapeResult.value = shapeRest || []
  for (const key in shapeId) {
    shapeId[key] = ''
  }
  shapeResult.value.forEach((item: any) => {
    shapeId[item.key] = item.id
  })
}

const handleChange = (standardRest: any, shapeRest: any) => {
  handleReset(standardRest, shapeRest)
  if (props.categorieResult) {
    if (props.categorieResult?.standards?.length > 0) {
      const standardsSet = new Set(props.categorieResult?.standards) // 转换为 Set 提高查找效率
      standardsRows.value = (state.standards || []).filter((row: any) => standardsSet.has(row.id))
    } else {
      standardsRows.value = []
    }
    if (props.categorieResult?.shapes?.length > 0) {
      const shapesSet = new Set(props.categorieResult?.shapes) // 转换为 Set 提高查找效率
      shapesRows.value = Object.keys(state.shapes).reduce((acc: any, key) => {
        // 过滤当前键对应的数组
        const filteredArray = state.shapes[key].filter((row: any) => shapesSet.has(row.id))
        // 如果过滤后的数组不为空，则保留该键
        if (filteredArray.length > 0) {
          acc[key] = filteredArray
        }
        return acc
      }, {})
      if (Object.keys(shapesRows.value).length < 4) isShapeExpanded.value = true
    } else {
      shapesRows.value = {}
    }
  } else {
    standardsRows.value = JSON.parse(JSON.stringify(state.standards))
    shapesRows.value = JSON.parse(JSON.stringify(state.shapes))
  }
}

// 暴露组件方法
defineExpose({ handleChange, handleReset })

const init = () => {
  keywords.value = ''
  fetchStandards()
  fetchShapes()
}

/* 生命周期钩子 */
onMounted(() => {
  init()
})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {})
</script>

<style lang="scss">
.el-popover.el-popper.shape-popover-box {
  width: 100px !important;
  min-width: 100px !important;
  padding: 5px 5px 10px !important;

  .shape-popover-content {
    width: 100%;
    margin: 0 auto;
    text-align: center;

    .shape-popover-title {
      width: 100%;
      text-align: center;
    }

    .shape-popover-image {
      width: 100%;
      height: auto;
      margin: 0 auto;
    }
  }
}

.products-filter-box {
  .standard-tag {
    width: 90%;
    height: 35px;
    max-height: 35px;
    overflow: hidden;

    .el-radio {
      margin: 0 10px 0 0;
    }

    &.isStandardExpanded {
      height: auto;
      max-height: 150px;
      transition:
        max-height 0.5s ease-in,
        height 0.5s ease-in;
    }

    &.standardCollapsed {
      animation: collapsedList 0.4s ease-in;
    }
  }

  .el-form-item {
    position: relative;
    align-items: flex-start;

    &.standard-form-box {
      .el-form-item__label {
        padding-top: 6px;
      }
    }

    &.shape-flex-form-item {
      .el-form-item__label {
        padding-top: 2px;
      }
    }

    .more-standard {
      position: absolute;
      top: 5px;
      right: 20px;
      z-index: 1000;
      display: flex;
      align-items: center;
      justify-content: flex-start;
      color: var(--default-color);
      cursor: pointer;
    }
  }

  .el-tag {
    &.active {
      color: #fff;
    }

    &.primary {
      color: var(--default-color);
    }
  }

  .shape-flex-radio-group {
    display: flex;
    flex-wrap: nowrap;
    align-items: flex-start;
    justify-content: flex-start;
    width: 100%;
  }

  .shape-checked {
    position: absolute;
    top: 2px;
    left: 0;
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    overflow: hidden;
    background-color: rgba(0, 0, 0, 0.2);

    .triangle {
      position: absolute;
      top: -10px;
      right: -18px;
      width: 40px;
      height: 24px;
      text-align: center;
      background: var(--default-color);
      transform: rotate(45deg);
    }
  }

  .el-icon.shape-checked-icon {
    margin-top: 12px;
    font-size: 10px;
    color: #fff;
    transform: rotate(-45deg);
  }

  .arrow-img {
    display: inline-block;
    margin-left: 3px;
    font-size: 18px;
    vertical-align: baseline;
    -o-transition: all 0.2s ease-in-out;
    -webkit-transition: all 0.2s ease-in-out;
    -moz-transition: all 0.2s ease-in-out;
    transition: all 0.2s ease-in-out;

    &.isRoute {
      -moz-transform: rotateZ(180deg);
      -webkit-transform: rotateZ(180deg);
      -o-transform: rotateZ(180deg);
      transform: rotateZ(180deg);
    }
  }

  @keyframes collapsedList {
    0% {
      height: auto;
      max-height: 150px;
    }

    100% {
      height: 35px;
      max-height: 35px;
    }
  }

  .shape-form-box {
    position: relative;
    width: 100%;
    height: 220px;
    max-height: 220px;
    overflow: hidden;

    .top-panel,
    .el-form,
    .shape-tag-box {
      width: 100%;
      height: 100%;

      .el-form-item {
        height: auto !important;
      }
    }

    &.shapeExpanded {
      height: auto;
      max-height: 800px;
      transition:
        max-height 0.5s ease-in,
        height 0.5s ease-in;
    }

    &.shapeCollapsed {
      animation: collapsedShapeAnimation 0.5s ease-in;
    }
  }

  .more-shape {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    margin: 0 auto;
    color: #fff;
    background-color: var(--default-color);
  }

  @keyframes collapsedShapeAnimation {
    0% {
      height: auto;
      max-height: 800px;
    }

    100% {
      height: 220px;
      max-height: 220px;
    }
  }
}
</style>
