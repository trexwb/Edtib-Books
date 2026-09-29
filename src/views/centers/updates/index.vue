<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/centers/updates/index.vue
 * @Description: 标准更新（订阅专属权益）：订阅全年可获取全部产品标准及当年标准更新；
 *   积分单独购买的标准不进入更新通道，始终保留购买时版本快照（见「我的已购标准」）。
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->

<template>
  <div v-loading="listLoading" class="list-container table-auto-height updates-box">
    <!-- 订阅权益说明 -->
    <el-card class="guide-card" shadow="never">
      <div class="guide-row">
        <div class="guide-main">
          <el-tag effect="dark" size="large" :type="subscribed ? 'success' : 'warning'">
            {{ subscribed ? '订阅生效中' : '未订阅' }}
          </el-tag>
          <div class="guide-info">
            <template v-if="subscribed">
              <div class="guide-title">订阅权益：全部产品标准 + 当年标准更新</div>
              <div class="guide-desc">更新窗口 {{ updateWindowText }}；下方为 {{ activeYear }} 年已发布的标准更新，可直接下载最新版本</div>
            </template>
            <template v-else>
              <div class="guide-title">标准更新为订阅专属权益</div>
              <div class="guide-desc">{{ guide || '订阅全年后可获取全部标准及当年标准更新（积分购买的标准不含后续更新）' }}</div>
            </template>
          </div>
        </div>
        <div class="guide-actions">
          <el-button :icon="List" @click="goPurchased">我的已购标准</el-button>
          <el-button v-if="!subscribed" :icon="VipCrown" type="primary" @click="goSubscription">订阅全年</el-button>
        </div>
      </div>
    </el-card>

    <!-- 年份 / 标准体系筛选 -->
    <div class="filter-row">
      <el-radio-group v-model="activeYear" @change="changeFilter">
        <el-radio-button v-for="year in years" :key="year" :label="year">{{ year }} 年</el-radio-button>
      </el-radio-group>
      <el-radio-group v-model="activeType" class="type-filter" @change="changeFilter">
        <el-radio-button label="">全部体系</el-radio-button>
        <el-radio-button v-for="item in typeOptions" :key="item.value" :label="item.value">{{ item.label }}</el-radio-button>
      </el-radio-group>
    </div>

    <el-table v-if="subscribed" v-loading="listLoading" :border="true" :data="rows.list" :stripe="true" style="width: 100%">
      <el-table-column align="center" label="标准体系" prop="type_name" width="120" />
      <el-table-column align="center" label="标准编号" prop="code" show-overflow-tooltip width="180" />
      <el-table-column align="center" label="标准名称" prop="name" show-overflow-tooltip />
      <el-table-column align="center" label="更新版本" prop="version" width="110" />
      <el-table-column align="center" label="更新说明" prop="title" show-overflow-tooltip />
      <el-table-column align="center" label="发布日期" prop="released_at" width="180">
        <template #default="{ row }">{{ row.released_at || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" fixed="right" label="操作" width="120">
        <template #default="{ row }">
          <el-button v-if="downloadUrl(row)" :icon="Download" link type="primary" @click="downloadUpdate(row)">下载</el-button>
          <span v-else class="no-file">暂无文件</span>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty class="vab-data-empty" description="本年度暂无标准更新" />
      </template>
    </el-table>

    <el-pagination
      v-if="subscribed && rows.total > queryForm.pageSize"
      background
      :current-page="queryForm.page"
      :layout="layout"
      :page-size="queryForm.pageSize"
      :total="rows.total"
      @current-change="handleCurrentChange"
    />

    <!-- 未订阅：订阅引导（服务端返回 subscribed=false 与 guide，非错误） -->
    <el-card v-if="!subscribed && !listLoading" class="empty-card" shadow="never">
      <el-empty :description="guide || '订阅全年后可获取全部标准及当年标准更新'">
        <el-button :icon="VipCrown" type="primary" @click="goSubscription">立即订阅</el-button>
      </el-empty>
      <div class="empty-tip">
        说明：订阅有效期内可获取全部产品标准及当年标准更新；通过积分单独购买的产品标准保留购买时版本，不随标准更新同步。
      </div>
    </el-card>
  </div>
</template>

<script lang="ts" setup>
// 说明：当前 icons 版本无 VipCrown 图标，以 GoldMedal 别名替代，模板引用名保持不变
import { Download, GoldMedal as VipCrown, List } from '@element-plus/icons-vue'
import { updatesList } from '/@/api/products'
import { getStorage } from '/@/utils/storage'
const $baseMessage = inject<any>('$baseMessage')

defineOptions({ name: 'CentersUpdates' })

const router = useRouter()
const listLoading = ref(false)
const subscribed = ref(false)
const guide = ref('')
const updateWindow = ref<any>({})
const years = ref<number[]>([new Date().getFullYear()])
const activeYear = ref<number>(new Date().getFullYear())
const activeType = ref<string | number>('')
const layout = ref('prev, pager, next, total')

/* 标准体系可选值：材料 / 性能 / 表面为免费内容，更新通道仅产品标准（products） */
const typeOptions = reactive([
  { label: '产品标准', value: 'products' },
  { label: '材料标准', value: 'materials' },
  { label: '性能标准', value: 'capabilities' },
  { label: '表面标准', value: 'exteriors' },
])

const queryForm = reactive({
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
})
const rows = reactive({
  total: 0,
  list: [] as any[],
})

const updateWindowText = computed(() => {
  const start = String(updateWindow.value?.start ?? '')
  const end = String(updateWindow.value?.end ?? '')
  return start && end ? `${start} ~ ${end}` : '当年'
})

const downloadUrl = (row: any) => String(row?.url ?? '')

const fetchData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await updatesList({
      year: activeYear.value,
      type: activeType.value === '' ? undefined : activeType.value,
      page: queryForm.page,
      pageSize: queryForm.pageSize,
    })
    subscribed.value = !!data?.subscribed
    guide.value = String(data?.guide ?? '')
    updateWindow.value = data?.update_window ?? {}
    const list = Array.isArray(data?.years) ? data.years.map((item: any) => Number(item)).filter((item: number) => !!item) : []
    if (list.length) years.value = list
    rows.list = Array.isArray(data?.list) ? data.list : []
    rows.total = Number(data?.total ?? rows.list.length)
  } catch (error) {
    console.error('[centersUpdates] 标准更新加载失败:', error)
    rows.list = []
    rows.total = 0
  } finally {
    listLoading.value = false
  }
}

const changeFilter = () => {
  queryForm.page = 1
  fetchData()
}

const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchData()
}

const downloadUpdate = async (row: any) => {
  const url = downloadUrl(row)
  if (!url) {
    $baseMessage?.('该更新暂无可下载文件，请前往标准详情查看最新版本', 'warning', 'hey')
    return
  }
  const link = document.createElement('a')
  link.setAttribute('href', url)
  link.setAttribute('download', String(row?.name || row?.code || 'standard-update'))
  link.setAttribute('target', '_blank')
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
}

const goSubscription = () => router.push({ path: '/centers/subscription' })
const goPurchased = () => router.push({ path: '/centers/purchased' })

onBeforeMount(() => {
  fetchData()
})
</script>

<style lang="scss">
.updates-box {
  background: rgb(255, 255, 255);

  .guide-card {
    margin-bottom: 16px;
  }

  .guide-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
  }

  .guide-main {
    display: flex;
    gap: 14px;
    align-items: center;
  }

  .guide-title {
    margin-bottom: 4px;
    font-size: 15px;
    font-weight: 600;
    color: #052965;
  }

  .guide-desc {
    font-size: 13px;
    color: #909399;
  }

  .filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    margin-bottom: 12px;
  }

  .type-filter {
    margin-left: 4px;
  }

  .no-file {
    font-size: 13px;
    color: #c0c4cc;
  }

  .empty-card {
    .empty-tip {
      padding-bottom: 12px;
      font-size: 13px;
      color: #909399;
      text-align: center;
    }
  }
}
</style>
