<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-14 11:42:02
 * @FilePath: /fastenerTradeWorkbench/Users/wbtrex/website/localServer/node/edtib/client/books/web/src/views/standards/materials/index.vue
 * @Description: 材料标准（免费内容）：注册用户全部可见，无积分购买 / 订阅门禁。
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->
<template>
  <div class="list-container table-auto-height materials-container">
    <!-- <el-alert
      class="free-alert"
      :closable="false"
      description="无需积分购买或订阅即可查看，支持牌号 / ISC / 旧牌号 / 名称检索"
      show-icon
      title="材料标准为免费内容，注册用户全部可见"
      type="success"
    /> -->

    <div class="filter-row">
      <el-input
        v-model="queryForm.filter.keyword"
        clearable
        placeholder="牌号 / ISC / 旧牌号 / 名称"
        style="width: 260px"
        @clear="handleSearch"
        @keyup.enter="handleSearch"
      />
      <el-select v-model="queryForm.filter.category_id" clearable placeholder="材料类别" style="width: 180px" @change="handleSearch">
        <el-option v-for="item in categoryOptions" :key="item.value" :label="item.label" :value="item.value" />
      </el-select>
      <el-button :icon="Search" type="primary" @click="handleSearch">查询</el-button>
      <el-button :icon="Refresh" @click="handleReset">重置</el-button>
    </div>

    <el-table v-loading="listLoading" :border="true" :data="rows.list" :stripe="true" style="width: 100%">
      <el-table-column align="center" label="牌号" prop="code" show-overflow-tooltip width="140" />
      <el-table-column align="center" label="ISC" width="110">
        <template #default="{ row }">{{ row.isc || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="旧牌号" width="120">
        <template #default="{ row }">{{ row.old_code || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="材料类别" width="140">
        <template #default="{ row }">{{ categoryName(row.category) }}</template>
      </el-table-column>
      <el-table-column align="center" label="对应标准" show-overflow-tooltip>
        <template #default="{ row }">{{ standardText(row) }}</template>
      </el-table-column>
      <el-table-column align="center" label="密度" width="100">
        <template #default="{ row }">{{ row.density ?? '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="访问" width="90">
        <template #default>
          <el-tag effect="dark" size="small" type="success">免费</el-tag>
        </template>
      </el-table-column>
      <el-table-column align="center" fixed="right" label="操作" width="90">
        <template #default="{ row }">
          <el-button link type="primary" @click="openDetail(row)">详情</el-button>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty class="vab-data-empty" description="暂无材料数据" />
      </template>
    </el-table>

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

    <el-drawer v-model="detailVisible" destroy-on-close size="88%" :title="detailTitle">
      <div v-loading="detailLoading" class="material-detail">
        <el-descriptions :border="true" :column="2" title="基本信息">
          <el-descriptions-item label="牌号">{{ detailRow.code || '—' }}</el-descriptions-item>
          <el-descriptions-item label="ISC">{{ detailRow.isc || '—' }}</el-descriptions-item>
          <el-descriptions-item label="旧牌号">{{ detailRow.old_code || '—' }}</el-descriptions-item>
          <el-descriptions-item label="别名">{{ listText(detailRow.alias) }}</el-descriptions-item>
          <el-descriptions-item label="材料类别">{{ categoryName(detailRow.category) }}</el-descriptions-item>
          <el-descriptions-item label="密度">{{ detailRow.density ?? '—' }}</el-descriptions-item>
          <el-descriptions-item label="对应标准">{{ standardText(detailRow) }}</el-descriptions-item>
          <el-descriptions-item label="特性">{{ listText(detailRow.property) }}</el-descriptions-item>
          <el-descriptions-item label="说明" :span="2">{{ detailRow.detail || '—' }}</el-descriptions-item>
        </el-descriptions>

        <div class="section-title">化学成分（%）</div>
        <!-- 横向表（对齐材数库）：列=元素，行=最小值/最大值；区间语义 ≤/≥/—；公式备注内嵌在对应元素列 -->
        <el-table
v-if="chemistryColumns.length" :border="true" class="chem-table" :data="chemistryRows"
          :stripe="true" style="width: 100%">
          <el-table-column align="center" fixed="left" label="成分" prop="label" width="90" />
          <el-table-column
v-for="col in chemistryColumns" :key="`chem-${col}`" align="center" :label="col"
            min-width="86">
            <template #default="{ row }">{{ row.cells[col] ?? '—' }}</template>
          </el-table-column>
          <template #empty>
            <el-empty class="vab-data-empty" description="暂无化学成分数据" />
          </template>
        </el-table>
        <el-empty v-else class="vab-data-empty" description="暂无化学成分数据" />

        <div class="section-title">近似牌号对照</div>
        <!-- 按分组手风琴渲染（默认展开第一组）；每组通用 15 体系列 + 其他（extra）列 -->
        <el-collapse v-if="equivalentGroups.length" v-model="activeGroups" class="equiv-collapse">
          <el-collapse-item
v-for="group in equivalentGroups" :key="group.name" :name="group.name"
            :title="`${group.name}（${group.rows.length}）`">
            <el-table :border="true" class="equiv-table" :data="group.rows" :stripe="true">
              <el-table-column
v-for="col in EQUIVALENT_COLUMNS" :key="`eq-${col.prop}`" align="center"
                :label="col.label" min-width="110">
                <template #default="{ row }">{{ row[col.prop] || '—' }}</template>
              </el-table-column>
              <el-table-column align="center" label="其他" min-width="130">
                <template #default="{ row }">{{ extraText(row.extra) }}</template>
              </el-table-column>
            </el-table>
          </el-collapse-item>
        </el-collapse>
        <el-empty v-else class="vab-data-empty" description="暂无近似牌号数据" />
      </div>
    </el-drawer>
  </div>
</template>

<script lang="ts" setup>
import { Refresh, Search } from '@element-plus/icons-vue'
import { materialsCategories, materialsDetail, materialsList } from '/@/api/materials'
import { getStorage } from '/@/utils/storage'
const $baseMessage = inject<any>('$baseMessage')

const templateName = 'StandardsMaterials'
defineOptions({
  name: templateName,
})

/** 多语言 JSON 文本取值（names/titles 为 { zh_cn: '...' } 结构） */
const jsonText = (value: any): string => {
  if (!value) return ''
  if (typeof value === 'string') return value
  return String(value.zh_cn || value.zh || Object.values(value)[0] || '')
}
const categoryName = (category: any): string => (category ? jsonText(category.names) || String(category.code || '') : '—')
const listText = (value: any): string => {
  if (!value) return '—'
  const list = Array.isArray(value) ? value : [value]
  const text = list
    .map((item) => jsonText(item))
    .filter((item) => !!item)
    .join('、')
  return text || '—'
}
const standardText = (row: any): string => {
  const standard = jsonText(row?.standard)
  const name = jsonText(row?.standard_name)
  const text = [standard, name].filter((item) => !!item).join(' ')
  return text || '—'
}

const listLoading = ref(false)
const layout = ref('sizes, prev, pager, next, jumper, total')
const categoryOptions = ref<Array<{ label: string; value: number }>>([])
const queryForm = reactive({
  filter: {
    keyword: '',
    category_id: null as number | null,
  },
  page: 1,
  pageSize: Number(getStorage('pageSize') || 20),
})
const rows = reactive({
  total: 0,
  list: [] as any[],
})

const detailVisible = ref(false)
const detailLoading = ref(false)
const detailRow = ref<any>({})
const detailTitle = computed(() => {
  const code = detailRow.value?.code
  return code ? `材料详情 · ${code}` : '材料详情'
})

/** 化学成分横向表：列 = 元素（按后端 sort 顺序），行 = 最小值 / 最大值；
 *  区间语义由两行表达（对齐材数库原版：仅上限=最小值行"—"、仅下限=最大值行空白、
 *  公式/条件备注内嵌最小值行） */
const chemistryColumns = computed<string[]>(() =>
  (detailRow.value?.chemistries || []).map((item: any) => String(item.element || '')).filter((e: string) => !!e),
)
const chemistryRows = computed(() => {
  const list = detailRow.value?.chemistries || []
  const minRow: any = { label: '最小值', cells: {} as Record<string, string> }
  const maxRow: any = { label: '最大值', cells: {} as Record<string, string> }
  for (const item of list) {
    const element = String(item.element || '')
    if (!element) continue
    const hasMin = item.min != null && item.min !== ''
    const hasMax = item.max != null && item.max !== ''
    minRow.cells[element] = hasMin ? String(item.min) : (item.note || '—')
    maxRow.cells[element] = hasMax ? String(item.max) : (hasMin || item.note ? '' : '—')
    if (hasMin && hasMax && item.note) {
      // 双边区间 + 备注：备注并入最小值单元格提示（表格无 tooltip 场景，拼在值后）
      minRow.cells[element] = `${item.min}（${item.note}）`
    }
  }
  return [minRow, maxRow]
})

/** 近似对照：通用 15 体系列（方案 5.5） */
const EQUIVALENT_COLUMNS = [
  { prop: 'gb', label: '中国GB' },
  { prop: 'isc', label: '中国ISC' },
  { prop: 'cns', label: '中国台湾CNS' },
  { prop: 'jis', label: '日本JIS' },
  { prop: 'ks', label: '韩国KS' },
  { prop: 'astm', label: '美国ASTM' },
  { prop: 'uns', label: '美国UNS' },
  { prop: 'iso', label: 'ISO' },
  { prop: 'din', label: '德国DIN' },
  { prop: 'wnr', label: '德国W-Nr.' },
  { prop: 'nf', label: '法国NF' },
  { prop: 'en', label: '欧标EN' },
  { prop: 'gost', label: '俄罗斯GOST' },
  { prop: 'ss', label: '瑞典SS' },
  { prop: 'bs', label: '英国BS' },
]

/** 近似对照：按分组聚合（保持后端 sort 顺序），默认展开第一组 */
const equivalentGroups = computed(() => {
  const list = detailRow.value?.equivalents || []
  const map = new Map<string, any[]>()
  for (const item of list) {
    const name = String(item.group || '').trim() || '未分组'
    if (!map.has(name)) map.set(name, [])
    map.get(name)!.push(item)
  }
  return [...map.entries()].map(([name, rows]) => ({ name, rows }))
})
const activeGroups = ref<string[]>([])

/** extra 文本化：对象/数组格式化展示，字符串原样（未列入 15 体系的国家/体系） */
const extraText = (extra: any): string => {
  if (extra == null || extra === '') return '—'
  if (typeof extra === 'object') {
    if (Array.isArray(extra)) return extra.map((item) => (typeof item === 'object' ? JSON.stringify(item) : String(item))).join('、')
    return Object.entries(extra)
      .map(([key, value]) => `${key}: ${value}`)
      .join('；')
  }
  return String(extra)
}

const fetchCategories = async () => {
  try {
    const res: any = await materialsCategories()
    const payload = res?.data ?? res
    const list = Array.isArray(payload) ? payload : (payload?.data ?? payload?.list ?? [])
    categoryOptions.value = (Array.isArray(list) ? list : [])
      .map((item: any) => ({
        label: jsonText(item?.names) || String(item?.code || item?.id || ''),
        value: Number(item?.id ?? 0),
      }))
      .filter((item: any) => !!item.value)
  } catch (error) {
    console.error('[standardsMaterials] 材料类别加载失败:', error)
  }
}

const fetchData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await materialsList({
      filter: {
        keyword: queryForm.filter.keyword || undefined,
        category_id: queryForm.filter.category_id ?? undefined,
      },
      page: queryForm.page,
      pageSize: queryForm.pageSize,
    })
    rows.list = Array.isArray(data?.list) ? data.list : []
    rows.total = Number(data?.total ?? rows.list.length)
  } catch (error) {
    console.error('[standardsMaterials] 材料列表加载失败:', error)
    rows.list = []
    rows.total = 0
  } finally {
    listLoading.value = false
  }
}

const openDetail = async (row: any) => {
  detailVisible.value = true
  detailLoading.value = true
  detailRow.value = { ...row }
  try {
    const { data }: any = await materialsDetail({ id: Number(row?.id ?? 0) })
    if (data) detailRow.value = data
    const groups = equivalentGroups.value
    activeGroups.value = groups.length ? [groups[0].name] : []
  } catch (error) {
    console.error('[standardsMaterials] 材料详情加载失败:', error)
    $baseMessage?.('材料详情加载失败，请稍后重试', 'warning', 'hey')
  } finally {
    detailLoading.value = false
  }
}

const handleSearch = () => {
  queryForm.page = 1
  fetchData()
}
const handleReset = () => {
  queryForm.filter.keyword = ''
  queryForm.filter.category_id = null
  handleSearch()
}
const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchData()
}

const handleSizeChange = (value: number) => {
  queryForm.pageSize = value
  queryForm.page = 1
  fetchData()
}

onBeforeMount(() => {
  fetchCategories()
  fetchData()
})
</script>

<style lang="scss">
.materials-container {
  background: rgb(255, 255, 255);

  .free-alert {
    margin-bottom: 12px;
  }


  .filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    margin-bottom: 12px;
  }

  .material-detail {
    .section-title {
      margin: 18px 0 10px;
      font-size: 15px;
      font-weight: 600;
      color: #052965;
    }

    .chem-table,
    .equiv-table {
      width: 100%;
    }

    .equiv-collapse {
      border-top: none;

      :deep(.el-collapse-item__header) {
        font-weight: 600;
      }
    }
  }
}
</style>
