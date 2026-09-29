<!--
 * @Author: ${git_name}
 * @Date: 2025-04-10 09:45:37
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-16 17:46:11
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsList.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="list-box">
    <el-alert
      v-if="subscribed === false"
      class="free-tier-alert"
      :closable="false"
      show-icon
      title="当前为未订阅状态，仅展示免费样例与您已购买的产品标准；订阅后可查看全部产品标准及最新更新，也可用 5 积分单独购买单个标准（仅保留购买时版本）。如需开通请联系客服 13216118255"
      type="warning"
    />
    <el-row v-loading="state.loading" :gutter="20" style="width: 100%">
      <template v-if="rows.total > 0">
        <el-col v-for="(row, index) in rows.list" :key="`product[${index}]`" :lg="12" :md="24" :sm="24" :xl="12" :xs="24">
          <el-card shadow="hover" style="width: 100%; cursor: pointer" @click="handleView(row)">
            <el-descriptions style="width: 100%">
              <el-descriptions-item :rowspan="3" :width="140">
                <el-image
                  fit="cover"
                  :src="row.covers[0]?.url ? `${row.covers[0].url}!a200` : productsPlaceholder"
                  style="width: 120px; height: 120px"
                >
                  <template #error>
                    <div class="image-slot">
                      <el-icon><icon-picture /></el-icon>
                    </div>
                  </template>
                </el-image>
              </el-descriptions-item>
              <el-descriptions-item :span="12">
                <div class="name-text">
                  <el-tag v-if="row.access_tag" class="free-tier-tag" effect="dark" size="small" :type="row.access_tag.type">
                    {{ row.access_tag.access === 'free' ? translate('免费') : row.access_tag.label }}
                  </el-tag>
                  <span
                    v-html="
                      highlightText(
                        `${row.standard}${!row.grade ? '' : '/'}${row.grade} ${row.code}${!row.year ? '' : '-'}${row.year}`,
                        'keywords'
                      )
                    "
                  ></span>
                </div>
              </el-descriptions-item>
              <el-descriptions-item v-if="row.names['zh-cn']" :span="12">
                <div v-html="highlightText(row.names['zh-cn'], 'keywords')"></div>
              </el-descriptions-item>
              <el-descriptions-item v-if="row.names['en']" :span="12">
                <div v-html="highlightText(row.names['en'], 'keywords')"></div>
              </el-descriptions-item>
            </el-descriptions>
            <!-- 积分购买入口（每标准默认 5 积分，二次确认后扣费；订阅有效期内无需购买） -->
            <div class="purchase-bar">
              <template v-if="row.access_tag && row.access_tag.access === 'paid'">
                <span class="price-text">{{ priceCreditOf(row) }} 积分</span>
                <el-button class="purchase-btn" plain size="small" type="warning" @click.stop="handlePurchase(row)">
                  {{ translate('购买后查看当前版本') }}
                </el-button>
              </template>
              <template v-else-if="row.access_tag && row.access_tag.access === 'purchased'">
                <span class="purchased-text">{{ translate('已购版本') }} {{ row.purchased_version || row.year || '-' }}</span>
                <span v-if="row.has_update" class="update-text">{{ translate('（有新版本，订阅后可获取更新）') }}</span>
              </template>
            </div>
          </el-card>
        </el-col>
      </template>
    </el-row>
    <template v-if="showEmpty">
      <el-empty description="没有找到对应的产品标准" />
    </template>
    <el-pagination
      v-if="rows.total > queryForm.pageSize"
      background
      :current-page="queryForm.page"
      :layout="layout"
      :page-size="queryForm.pageSize"
      :total="rows.total"
      @current-change="handleCurrentChange"
      @size-change="handleSizeChange"
    />
    <products-detail ref="viewRef" />
  </div>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { productsList, productsPurchase } from '/@/api/products'
import { translate } from '/@/i18n'
import { ElMessageBox } from 'element-plus'
import { useRouter } from 'vue-router'
import { useUserStore } from '/@/store/modules/user'
import { Picture as IconPicture } from '@element-plus/icons-vue'
import productsPlaceholder from '/@/assets/products_placeholder.svg'

/* 组件模板名称 */
const templateName = 'StandardsProductsList'

/* 定义组件选项 */
defineOptions({
  name: templateName, // 注册组件名称，用于调试和keep-alive缓存标识
})

const emit = defineEmits(['categories-change']) // 定义组件事件

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用
const route = useRoute()
const router = useRouter()

/* 核心响应式状态 */
// 访问态徽标映射（与服务端 access 四态对齐）：
// free=免费样例 / member=订阅有效（全部可见 + 最新更新）/ purchased=积分单购（仅购买时版本）/ paid=未购收费内容
const accessTag = (row: any): { access: string; label: string; type: 'success' | 'primary' | 'warning' | 'info' } => {
  const access = String(row?.access || (Number(row?.free_tier ?? 0) === 1 ? 'free' : 'paid'))
  if (access === 'free') return { access, label: '免费', type: 'success' }
  if (access === 'member') return { access, label: '会员', type: 'primary' }
  if (access === 'purchased') return { access, label: '已购', type: 'success' }
  return { access, label: '收费', type: 'warning' } // paid
}

// 单购价格：服务端 products.extension.price_credit 可覆盖，默认 5 积分/标准
const priceCreditOf = (row: any): number => Number(row?.price_credit ?? 5) || 5

const userStore = useUserStore()

const state = reactive<any>({
  loading: false,
  operationFixed: true,
  selectionRows: [],
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
})

const rows = reactive({
  total: 0,
  list: [] as any[],
})

const viewRef = ref<any>(null)
const showEmpty = ref(false)
const $baseMessage = inject<any>('$baseMessage')
/* 订阅状态：由 productsList 出口下发（IPC 本地库无该字段，视为已订阅不展示引导） */
const subscribed = ref<boolean>(true)

const handleView = (item: any) => {
  // 未订阅用户仅可打开「免费可见」与「已购」的产品标准；离线 IPC 直连本地库无服务端裁剪，入口处兜底拦截
  const access = String(item?.access || (Number(item?.free_tier ?? 0) === 1 ? 'free' : 'paid'))
  if (subscribed.value === false && access === 'paid') {
    handlePurchase(item)
    return
  }
  viewRef.value.showView(item)
}

/**
 * 积分购买单个产品标准（默认 5 积分）：二次确认 → 调用后端事务扣费 → 成功刷新为已购态
 * 幂等：订阅有效期内或已购项后端不再扣费（返回 charged=false），前端按返回 access 直接放行
 */
const handlePurchase = async (item: any) => {
  const price = priceCreditOf(item)
  try {
    await ElMessageBox.confirm(
      translate(`是否使用 ${price} 积分购买该产品标准？购买后可永久查看当前版本内容；订阅有效期内可获取全部标准及最新更新。`),
      translate('积分购买'),
      {
        confirmButtonText: translate('确认购买'),
        cancelButtonText: translate('暂不购买'),
        type: 'warning',
        draggable: false,
      }
    )
  } catch {
    return // 用户取消
  }
  try {
    const { data } = (await productsPurchase({ id: Number(item.id) })) as any
    // 余额同步：服务端已同步 customer_users.credit，这里刷新本地展示
    if (typeof data?.balance === 'number') userStore.setCredit(data.balance)
    $baseMessage?.(translate('购买成功，已解锁当前版本'), 'success', 'hey')
    await fetchData()
    // 购买后重新拉取到 purchased 态，直接打开详情
    const row = (rows.list || []).find((it: any) => Number(it.id) === Number(item.id)) || item
    viewRef.value.showView(row)
  } catch (error: any) {
    // 400400010002：积分余额不足，引导充值；其余错误由 request 拦截器已提示
    if (Number(error?.code ?? 0) === 400400010002) {
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
    console.error('[productsList] 积分购买失败:', error)
  }
}

const props = defineProps({
  queryForm: {
    type: Object,
    // 对象/数组类型的 default 必须使用工厂函数，避免多实例共享同一引用
    default: () => ({
      filter: {
        keywords: '',
        id: null as number | string | null,
        standard_id: null as number | string | null,
        shape_id: {} as Record<string, string>,
        category_id: null as number | string | null,
        code: '',
        status: '',
      },
      page: 1,
      pageSize: 20,
      sort: '-id',
    }),
  },
})

// 定义 target 的合法类型
type FilterKeys = keyof typeof props.queryForm.filter
// HTML 转义：防止命中内容经 v-html 渲染时产生 XSS
const escapeHtml = (s: string): string =>
  s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] as string)
// 正则元字符转义：防止搜索词含 ( [ \ 等导致 new RegExp 抛 SyntaxError
const escapeRegExp = (s: string): string => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
// 修改 highlightText 方法，确保 target 类型安全
const highlightText = (text: any, target: FilterKeys) => {
  // 检查 target 是否为合法键
  if (!(target in props.queryForm.filter)) {
    // C-B7: 安全返回路径也必须转义，否则原始数据可注入 HTML（v-html 渲染出口）
    return escapeHtml(String(text))
  }
  const keyword = props.queryForm.filter[target]
  if (keyword === '' || keyword == null) {
    return escapeHtml(String(text))
  }
  const escapedText = escapeHtml(String(text))
  const safeKeyword = escapeRegExp(String(keyword))
  return escapedText.replace(new RegExp(safeKeyword, 'g'), (matched) => `<span style="color:#F56C6C">${matched}</span>`)
}

const layout = ref('sizes, prev, pager, next, jumper, total')

const fetchData = async () => {
  rows.list = []
  try {
    if (state.loading) return
    state.loading = true
    const { data }: any = await productsList(props.queryForm)
    // 确保 data 和 data.list 存在且是数组；服务端下发 access（free/member/purchased/paid）与 price_credit，本地仅做展示兜底
    rows.list = (Array.isArray(data?.list) ? data.list : []).map((item: any) => ({
      ...item,
      price_credit: priceCreditOf(item),
      access: String(item?.access || (Number(item?.free_tier ?? 0) === 1 ? 'free' : 'paid')),
      access_tag: accessTag(item),
    }))
    rows.total = data?.total || 0
    // 订阅状态由服务端下发（未订阅时服务端已按 free_tier=1 裁剪列表）；IPC 直连本地库无该字段，保持默认 true
    if (typeof data?.subscribed === 'boolean') subscribed.value = data.subscribed
    state.loading = false
    showEmpty.value = rows.total > 0 ? false : true
  } catch (error) {
    console.error('Fetch data error:', error)
  }
}

// 页码改变
const handleSizeChange = (value: number) => {
  props.queryForm.pageSize = value
  props.queryForm.page = 1
  fetchData()
}

const handleCurrentChange = (value: number) => {
  props.queryForm.page = value
  fetchData()
}

// 暴露组件方法
defineExpose({ fetchData })

const init = async () => {
  await fetchData()
  if (route.query.id) {
    const row = (rows.list || []).find((item) => item.id === Number(route.query.id))
    row?.id && handleView(row)
  }
}

/* 生命周期钩子 */
onMounted(() => {
  init()
})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {
  init()
})
</script>

<style lang="scss">
.free-tier-alert {
  margin-bottom: 10px;
}

.free-tier-tag {
  margin-right: 6px;
  vertical-align: middle;
}

/* 积分购买栏：未购项展示价格与购买按钮；已购项展示快照版本与更新提示 */
.purchase-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: flex-end;
  min-height: 24px;
  padding-top: 6px;
  border-top: 1px dashed var(--el-border-color-lighter);

  .price-text {
    font-size: 13px;
    font-weight: bold;
    color: #f56c6c;
  }

  .purchased-text {
    font-size: 13px;
    color: var(--default-color);
  }

  .update-text {
    font-size: 12px;
    color: #e6a23c;
  }
}

.card-header {
  display: flex;
  align-items: center;
  min-height: 22px;

  span {
    font-size: 16px;
  }
}

.el-tree-node {
  padding: 8px 0;
}

.custom-scrollbar .el-scrollbar__wrap {
  width: 100%;
  overflow-x: hidden !important;
  overflow-y: auto !important;
}

.custom-tree-node {
  display: flex;
  gap: 5px;
  align-items: center;
  font-size: 13px;
}

.tree-node-popover {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;

  .text {
    padding-bottom: 4px;
    font-size: 16px;
    font-weight: bold;
    color: var(--default-color);
  }
}

.custom-tree-node .el-icon {
  font-size: 16px;
  color: var(--default-color);
}

.node-label {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-tree-node:hover {
  // background-color: #f5f7fa;
  cursor: pointer;
}

.el-tree-node__expand-icon {
  font-size: 14px;
  color: var(--default-color);
}

.el-tree--highlight-current .el-tree-node.is-current > .el-tree-node__content {
  // background-color: rgba(45, 105, 105, 0.1);
  color: var(--default-color);
}

.el-tree-node__expand-icon {
  font-size: 20px;
}
</style>
