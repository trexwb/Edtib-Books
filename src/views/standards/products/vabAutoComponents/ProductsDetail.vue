<!--
 * @Author: ${git_name}
 * @Date: 2025-04-18 16:33:00
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-08-19 10:54:26
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetail.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <el-drawer
    v-model="state.drawerdFormVisible"
    append-to-body
    :before-close="handleClose"
    direction="rtl"
    :loading="state.loading"
    modal-class="products-detail-drawer"
    :show-close="false"
    size="88%"
    :title="state.title"
  >
    <template #header="{ close, titleId, titleClass }">
      <h2 :id="titleId" class="name-text" :class="titleClass">
        <el-tag
          v-if="productRow.access === 'free' || Number(productRow.free_tier ?? 0) === 1"
          class="free-tier-tag"
          effect="dark"
          size="small"
          type="warning"
        >
          {{ translate('免费') }}
        </el-tag>
        <el-tag v-else-if="productRow.access === 'purchased'" class="free-tier-tag" effect="dark" size="small" type="success">
          {{ translate('已购') }}
        </el-tag>
        <el-tag v-else-if="productRow.access === 'member'" class="free-tier-tag" effect="dark" size="small" type="primary">
          {{ translate('会员') }}
        </el-tag>
        {{ `${productRow.standard}${!productRow.grade ? '' : '/'}${productRow.grade}` }}
        {{ `${productRow.code}${!productRow.year ? '' : '-'}${productRow.year}` }}
        {{ productRow.names?.[state.defaultLang] || '' }}
      </h2>
      <el-icon size="24" style="padding-bottom: 15px" @click="close">
        <close />
      </el-icon>
    </template>
    <section v-if="state.drawerdFormVisible" class="products-detail-drawer-content">
      <!-- 积分单购版本锁定提示：购买用户仅保留购买时版本，产品后续更新不向已购用户同步 -->
      <el-alert
        v-if="productRow.version_locked"
        class="version-locked-alert"
        :closable="false"
        show-icon
        :title="`当前为积分购买版本 ${productRow.purchased_version || ''}${productRow.latest_version && productRow.latest_version !== productRow.purchased_version ? '，该标准已有更新版本 ' + productRow.latest_version + '，订阅后可获取全部标准及最新更新' : '，订阅后可获取全部标准及最新更新'}`"
        type="info"
      />
      <products-detail-basic
        ref="detailBasicRef"
        :product-row="productRow"
        :query-form="queryForm"
        :variables="variables"
        @update-length="updateLength"
        @update-query-form="updateQueryForm"
      />
      <products-detail-formulas
        ref="detailFormulasRef"
        :formulas="formulas"
        :product-row="productRow"
        :query-form="queryForm"
        :shape-parts="shapeParts"
        :variables="variables"
      />
      <products-detail-detail v-if="!!productRow.detail[state.defaultLang]" ref="detailDetailRef" :product-row="productRow" />
      <products-detail-interpretations
        v-if="productRow.interpretation?.id"
        ref="detailInterpretationsRef"
        :content="productRow.interpretation"
      />
    </section>
    <div style="height: 80px"></div>
    <template #footer>
      <div style="flex: auto">
        <el-button @click="close">关闭</el-button>
      </div>
    </template>
  </el-drawer>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { variablesAll } from '/@/api/variables'
import { formulasAll } from '/@/api/formulas'
import { productsDetail, productsPurchase, shapesAll, requestBridge } from '/@/api/products'
import { translate } from '/@/i18n'
import { ElLoading, ElMessageBox } from 'element-plus'
import { getVariables } from '/@/utils/evaluateCondition'
import { useUserStore } from '/@/store/modules/user'
import { Close } from '@element-plus/icons-vue'

const templateName = 'StandardsProductsDetail'
defineOptions({
  name: templateName,
})

const $baseLoading = inject<any>('$baseLoading')
const $baseMessage = inject<any>('$baseMessage')

const detailBasicRef = ref<any>(null)
const detailFormulasRef = ref<any>(null)
const detailDetailRef = ref<any>(null)
const detailInterpretationsRef = ref<any>(null)

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { languages, defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用
const route = useRoute()
const router = useRouter() // 路由实例
const userStore = useUserStore() // 积分余额展示/同步

const state = reactive<any>({
  drawerdFormVisible: false,
  activeName: defaultItem.value.languages || 'zh-cn', // 当前激活的标签页
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
  loading: false,
  title: '',
})

// 筛选值集合
const queryForm = ref<any>({})
const sideNavigationTabsActive = ref(`${route.query.id ? 'Interpret' : 'Basic'}`)
const productRow = ref<any>({})
const variables = ref<any>({})
const formulas = ref<any>({})
/* formulas 表原始行（含 shape_id），用于按形状装配体积公式 */
const formulaRows = ref<any[]>([])
/* 产品关联形状（体积公式取自 formulas 表 shape_id 关联），用于体积求和 */
const shapeParts = ref<any[]>([])

// IPC 直读本地库时 JSON 列可能以字符串形态返回，统一归一为对象（解析时告警，失败信息可见）
const JSON_FIELDS = [
  'names',
  'detail',
  'parameters',
  'tolerance',
  'diameterLength',
  'drawingLimit',
  'formulas',
  'extension',
  'covers',
  'svgs',
  'renders',
  'cads',
  'assemblies',
  'models',
]
const normalizeProductRow = (row: any) => {
  if (!row || typeof row !== 'object') return row
  JSON_FIELDS.forEach((key) => {
    const value = row[key]
    if (typeof value === 'string' && value.trim()) {
      try {
        row[key] = JSON.parse(value)
        console.warn(`[productsDetail] ${key} 为字符串形态，已自动解析（IPC 本地库数据）`)
      } catch (e) {
        console.error(`[productsDetail] ${key} JSON 解析失败:`, value.slice(0, 80), e)
      }
    }
  })
  return row
}

const fetchData = async (id: any) => {
  if (state.loading) return
  state.loading = true
  const { data } = await productsDetail({ id: id || 0 })
  state.loading = false
  return normalizeProductRow(data)
}

const fetchVariablesData = async () => {
  if (state.loading) return
  variables.value = {}
  state.loading = true
  const { data } = await variablesAll()
  state.loading = false
  // 后端 variablesAll 直接返回数组（非 { list } 包裹），IPC 亦返回数组，统一兼容两种结构
  const variablesList = Array.isArray(data) ? data : data?.list || []
  variablesList.map((item: any) => {
    variables.value[item.variable] = item
  })
}

/* 关联形状：HTTP 详情自带 shapes_id；离线 IPC 优先从 products_filter 读取，旧库回退 products_shapes */
const fetchShapePartsData = async () => {
  shapeParts.value = []
  const productId = productRow.value?.id
  if (!productId) return
  let shapeIds: number[] = Array.isArray(productRow.value?.shapes_id) ? productRow.value.shapes_id.map(Number) : []
  if (!shapeIds.length) {
    try {
      const result = await requestBridge.getList({
        ipc: { table: 'products_filter', filters: { product_id: Number(productId), product_type: 0 }, limit: 100, offset: 0 },
      })
      shapeIds = (result?.list || []).filter((row: any) => Number(row.shape_id) > 0).map((row: any) => Number(row.shape_id))
    } catch {
      shapeIds = []
    }
    if (!shapeIds.length) {
      // 兼容旧库（未跑迁移前）：products_shapes 为历史直接关联表
      try {
        const legacy = await requestBridge.getList({
          ipc: { table: 'products_shapes', filters: { product_id: Number(productId) }, limit: 100, offset: 0 },
        })
        shapeIds = (legacy?.list || []).map((row: any) => Number(row.shape_id)).filter(Boolean)
      } catch {
        shapeIds = []
      }
    }
  }
  if (!shapeIds.length) return
  const { data } = await shapesAll()
  // HTTP 返回按位置分组的对象，IPC 返回行数组，统一摊平
  const flat: any[] = Array.isArray(data) ? data : Object.values(data || {}).flat()
  shapeParts.value = shapeIds
    .map((sid: number) => {
      const shape: any = flat.find((item: any) => Number(item?.id) === Number(sid))
      if (!shape) return null
      // 体积公式判定：formulas 表中 type=1（产品标准）且 shape_id 关联该形状的公式
      const formulas = formulaRows.value
        .filter((item: any) => item.type === 1 && Number(item.shape_id) === Number(sid))
        .sort((a: any, b: any) => (a.sort || 0) - (b.sort || 0))
      return { id: shape.id, location: shape.location, names: shape.names, formulas }
    })
    .filter(Boolean)
}

const fetchFormulasData = async () => {
  if (state.loading) return
  formulas.value = {}
  state.loading = true
  const { data } = await formulasAll()
  state.loading = false
  // 后端 formulasAll 直接返回数组（非 { list } 包裹），IPC 亦返回数组，统一兼容两种结构
  const formulasList = Array.isArray(data) ? data : data?.list || []
  formulaRows.value = formulasList
  formulasList.map((item: any) => {
    formulas.value[item.code] = item.columnar
  })
}

/**
 * 处理对话框关闭逻辑
 * @description 包含未保存修改确认流程
 */
const updateQueryForm = async (data: any) => {
  queryForm.value = {
    mon: data.mon || '', //公称名
    diameter: data.diameter || 0, // 直径
    diameterLength: data.diameterLength || 0, // 长度
    pitch: data.pitch || '', // 螺距
    variables: {},
  }
  if (data.mon) {
    queryForm.value.variables = getVariables(productRow.value.parameters || {}, productRow.value.tolerance || {}, queryForm.value)
  }
  await nextTick()
  detailBasicRef.value?.handelDefault()
  detailFormulasRef.value?.handelDefault()
  detailDetailRef.value?.handelDefault()
  detailInterpretationsRef.value?.handelDefault()
}

const updateLength = (searchLen: number) => {
  queryForm.value.diameterLength = Number(searchLen)
}

const showView = async (row: any) => {
  const loading = $baseLoading()
  try {
    updateQueryForm({})
    productRow.value = await fetchData(row.id)
    state.title = `${productRow.value.standard}${!productRow.value.grade ? '' : '/'}${productRow.value.grade} ${productRow.value.code}${!productRow.value.year ? '' : '-'}${productRow.value.year} ${productRow.value.names[state.defaultLang] || ''}`
    // 变量与公式相互独立，并行加载；形状公式装配依赖公式结果，保持其后执行
    await Promise.all([fetchVariablesData(), fetchFormulasData()])
    await fetchShapePartsData()
    sideNavigationTabsActive.value = 'Basic' // `${route.query.id ? 'Interpret' : 'Basic'}`
    await open()
    await nextTick()
  } catch (error) {
    console.error('[productsDetail] 详情加载失败:', error)
    // 403030005013：免费样例范围之外的产品标准（未订阅用户越权访问）。
    // request 拦截器已弹出后端 msg，这里提供积分购买引导（每标准 5 积分；购买后可永久查看当前版本，订阅可获取更新）
    if (Number((error as any)?.code ?? 0) === 403030005013) {
      try {
        await ElMessageBox.confirm(
          translate('该产品标准为收费内容，是否使用积分购买？购买后可永久查看当前版本，订阅用户可获取最新更新。'),
          translate('积分购买'),
          {
            confirmButtonText: translate('确认购买'),
            cancelButtonText: translate('暂不购买'),
            type: 'warning',
            draggable: false,
          }
        )
      } catch {
        return // 用户取消购买
      }
      try {
        const { data }: any = await productsPurchase({ id: row.id })
        // 余额同步：服务端已同步 customer_users.credit，这里刷新本地展示
        if (typeof data?.balance === 'number') userStore.setCredit(data.balance)
        $baseMessage(translate('购买成功，已解锁当前版本'), 'success', 'hey')
        row.purchased = true
        await showView(row)
      } catch (purchaseError: any) {
        console.error('[productsDetail] 积分购买失败:', purchaseError)
        // 400400010002：积分余额不足，引导前往积分中心充值
        if (Number(purchaseError?.code ?? 0) === 400400010002) {
          try {
            await ElMessageBox.confirm(
              translate('积分余额不足，可前往「积分中心」充值后继续购买，或联系客服 13216118255。'),
              translate('余额不足'),
              {
                confirmButtonText: translate('前往积分中心'),
                cancelButtonText: translate('稍后再说'),
                type: 'warning',
                draggable: false,
              }
            )
            await router.push({ path: '/centers/credits' })
          } catch {
            /* 用户放弃充值 */
          }
        }
      }
      return
    }
    $baseMessage('详情加载失败，请检查网关与数据库连接后重试', 'warning', 'hey')
  } finally {
    // 兜底关闭全屏加载遮罩：任一环节失败时若不关闭，遮罩会随多次打开逐层堆叠导致卡顿
    loading.close()
  }
}
// 对话框控制方法
const open = async () => {
  state.drawerdFormVisible = true
  await nextTick()
  detailBasicRef.value?.handelDefault()
  detailFormulasRef.value?.handelDefault()
  detailDetailRef.value?.handelDefault()
  detailInterpretationsRef.value?.handelDefault()
}
const close = async () => {
  if (route.query.id) {
    const query = { ...route.query } // 复制当前查询参数
    if (query['id']) {
      delete query['id'] // 删除指定参数
    }
    router.replace({ query })
  }
  state.drawerdFormVisible = false
}
// 关闭
const handleClose = () => {
  // detailBasicRef.value && typeof detailBasicRef.value.destroy === 'function' && detailBasicRef.value.destroy()
  close()
  detailBasicRef.value = null
  detailFormulasRef.value = null
  detailDetailRef.value = null
  detailInterpretationsRef.value = null
}

// 暴露组件方法
defineExpose({ showView })

/* 生命周期钩子 */
onMounted(() => {
  /* 组件挂载后逻辑 */
})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {
  /* 组件挂载前逻辑 */
})
</script>

<style lang="scss">
.el-overlay.products-detail-drawer {
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

  .el-switch__core {
    height: 23px;
  }

  .under-tabs-select-box {
    width: 100%;
    padding-bottom: 10px;
    background: var(--default-color);

    .el-form-item__label {
      div {
        color: #fff;
      }
    }

    .under-tabs-img-box {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 32px;
      background: #fff;
      border-right: 1px solid var(--default-color);
      border-top-left-radius: 3px;
      border-bottom-left-radius: 3px;

      div {
        font-size: 14px;
        color: var(--default-color);
      }

      img {
        width: auto;
        height: 23px;
        margin-left: 5px;
      }
    }

    .under-tabs-select-input {
      display: flex;
      flex: 0 0 auto;
      align-items: center;
      height: 40px;
      padding-top: 12px;
      line-height: 40px;

      .unit-text {
        margin-left: 6px;
        color: #fff;
        white-space: nowrap;
      }

      .total-value {
        width: calc(100% - 48px);
        height: 30px;
        padding: 0 5px;
        font-size: 15px;
        line-height: 30px;
        color: var(--default-color);
        background: #fff;
        border-radius: 4px;
      }
    }

    .notice-title {
      display: flex;
      flex-wrap: nowrap;
      align-items: center;
      justify-content: flex-end;
      width: 100%;
      padding-top: 15px;
      margin-left: 90px;

      .label-title {
        padding-left: 3px;
        color: #fff;
        white-space: nowrap;
      }
    }
  }

  .el-select__wrapper {
    border-radius: 3px !important;
  }

  .custom-el-form {
    .el-select__wrapper {
      border-top-left-radius: 0 !important;
      border-bottom-left-radius: 0 !important;
      box-shadow: none;
    }

    .el-form-item__label {
      padding: 0 !important;
    }

    .el-form-item {
      margin-bottom: 0 !important;
    }
  }

  .el-drawer__header {
    margin-bottom: 0 !important;
  }

  .products-detail-drawer-content {
    padding-bottom: 50px;
  }

  .products-detail-drawer-content,
  .el-row {
    width: 100%;
    height: 100%;
  }

  .el-tabs__item {
    justify-content: center !important;
  }

  .drawer-left-menu {
    width: 100%;

    div {
      width: 100%;
      height: 45px;
      line-height: 45px;
      text-align: center;

      &.active {
        color: #000;
        background: #eaf0f0;
        border-left: 3px solid var(--default-color);
      }
    }
  }

  h2.name-text {
    padding-bottom: 15px;
    font-size: 20px;
    color: var(--default-color);

    .free-tier-tag {
      margin-right: 6px;
      vertical-align: middle;
    }
  }

  .en-name {
    padding-bottom: 20px;
  }

  .tabs-label {
    font-size: 13px;
    font-weight: bold;
  }

  .el-tabs__item {
    width: 120px;
    padding: 0;

    &.is-active {
      background: #eaf0f0;
    }
  }
}
</style>
