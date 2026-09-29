<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/centers/purchased/index.vue
 * @Description: 我的已购标准：积分单独购买的产品标准清单（每标准 5 积分）；
 *   已购标准保留购买时版本快照，标准后续更新不向已购用户同步，仅订阅用户可获取全部标准及当年更新。
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->

<template>
  <div v-loading="listLoading" class="list-container table-auto-height purchased-box">
    <!-- 订阅权益说明 -->
    <el-card class="guide-card" shadow="never">
      <div class="guide-row">
        <div class="guide-main">
          <el-tag effect="dark" size="large" :type="subscribed ? 'success' : 'warning'">
            {{ subscribed ? '订阅生效中' : '未订阅' }}
          </el-tag>
          <div class="guide-info">
            <template v-if="subscribed">
              <div class="guide-title">订阅有效期内可获取全部产品标准及当年标准更新</div>
              <div class="guide-desc">下方为标准积分单购记录，保留购买时版本快照；最新标准更新请前往「标准更新」查看</div>
            </template>
            <template v-else>
              <div class="guide-title">积分购买的标准保留购买时版本，不随标准更新同步</div>
              <div class="guide-desc">{{ guide || '订阅全年可解锁全部标准及当年更新' }}</div>
            </template>
          </div>
        </div>
        <div class="guide-actions">
          <el-button :icon="Refresh" @click="goUpdates">标准更新</el-button>
          <el-button v-if="!subscribed" :icon="Wallet" type="primary" @click="goSubscription">订阅全年</el-button>
        </div>
      </div>
    </el-card>

    <!-- 标准体系筛选 -->
    <el-radio-group v-if="typeOptions.length > 1" v-model="activeType" class="type-filter" @change="changeType">
      <el-radio-button label="">全部体系</el-radio-button>
      <el-radio-button v-for="item in typeOptions" :key="item.value" :label="item.value">{{ item.label }}</el-radio-button>
    </el-radio-group>

    <el-table v-loading="listLoading" :border="true" :data="pageList" :stripe="true" style="width: 100%">
      <el-table-column align="center" label="标准体系" prop="type_name" width="120" />
      <el-table-column align="center" label="标准编号" prop="code" show-overflow-tooltip width="180" />
      <el-table-column align="center" label="标准名称" prop="name" show-overflow-tooltip />
      <el-table-column align="center" label="购买版本" width="110">
        <template #default="{ row }">{{ row.version || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="当前最新版本" width="130">
        <template #default="{ row }">{{ row.current_version || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="更新状态" width="140">
        <template #default="{ row }">
          <el-tag v-if="row.has_update" effect="dark" type="warning">有更新（订阅可获取）</el-tag>
          <el-tag v-else-if="subscribed" type="success">订阅可同步最新</el-tag>
          <el-tag v-else type="info">购买版本</el-tag>
        </template>
      </el-table-column>
      <el-table-column align="center" label="购买时间" prop="purchased_at" width="180">
        <template #default="{ row }">{{ row.purchased_at || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" fixed="right" label="操作" width="140">
        <template #default="{ row }">
          <el-button
            :icon="Download"
            link
            :loading="downloadingId === Number(row.standard_id)"
            type="primary"
            @click="downloadStandard(row)"
          >
            下载
          </el-button>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty class="vab-data-empty" description="暂无已购标准" />
      </template>
    </el-table>

    <el-pagination
      v-if="filteredList.length > queryForm.pageSize"
      background
      :current-page="queryForm.page"
      :layout="layout"
      :page-size="queryForm.pageSize"
      :total="filteredList.length"
      @current-change="handleCurrentChange"
    />

    <div v-if="!listLoading && filteredList.length === 0" class="empty-actions">
      <el-button type="primary" @click="goProducts">去标准查询选购（每标准 5 积分）</el-button>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Download, Refresh, Wallet } from '@element-plus/icons-vue'
import { purchasedList, productsDownloadPdfUrl } from '/@/api/products'
import { getStorage } from '/@/utils/storage'
const $baseMessage = inject<any>('$baseMessage')

defineOptions({ name: 'CentersPurchased' })

const router = useRouter()
const listLoading = ref(false)
const subscribed = ref(false)
const guide = ref('')
const records = ref<any[]>([])
const activeType = ref<string | number>('')
const downloadingId = ref(0)
const layout = ref('prev, pager, next, total')

const queryForm = reactive({
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
})

/* 服务端一次性返回全部已购记录（含 groups 分组），前端按标准体系筛选 + 本地分页 */
const typeOptions = computed(() => {
  const map = new Map<string, string>()
  records.value.forEach((row: any) => {
    const key = String(row?.standard_type ?? '')
    if (key) map.set(key, String(row?.type_name || key))
  })
  return Array.from(map.entries()).map(([value, label]) => ({ value, label }))
})
const filteredList = computed(() =>
  activeType.value === '' ? records.value : records.value.filter((row: any) => String(row?.standard_type) === String(activeType.value))
)
const pageList = computed(() => {
  const start = (queryForm.page - 1) * queryForm.pageSize
  return filteredList.value.slice(start, start + queryForm.pageSize)
})

const fetchData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await purchasedList({})
    records.value = Array.isArray(data?.list) ? data.list : []
    subscribed.value = !!data?.subscribed
    guide.value = String(data?.guide ?? '')
  } catch (error) {
    console.error('[centersPurchased] 已购标准加载失败:', error)
    records.value = []
  } finally {
    listLoading.value = false
  }
}

const changeType = () => {
  queryForm.page = 1
}

const handleCurrentChange = (value: number) => {
  queryForm.page = value
}

/* 下载：订阅中取最新版本，未订阅取购买时快照版本（服务端三态裁剪） */
const downloadStandard = async (row: any) => {
  const id = Number(row?.standard_id || row?.product_id || 0)
  if (!id) return
  downloadingId.value = id
  try {
    const { data }: any = await productsDownloadPdfUrl({ id })
    const url = String(data?.url || data?.path || '')
    if (!url) {
      $baseMessage?.('下载地址获取失败，请稍后重试', 'warning', 'hey')
      return
    }
    const link = document.createElement('a')
    link.setAttribute('href', url)
    link.setAttribute('download', String(row?.name || row?.code || `standard-${id}`))
    link.setAttribute('target', '_blank')
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (error: any) {
    console.error('[centersPurchased] 标准下载失败:', error)
    const code = Number(error?.code ?? 0)
    if (code === 403030005016) {
      $baseMessage?.('该标准暂不可下载，订阅全年可获取全部标准', 'warning', 'hey')
    } else if (code === 400200010002) {
      $baseMessage?.('该标准暂无 PDF 文件', 'warning', 'hey')
    }
  } finally {
    downloadingId.value = 0
  }
}

const goProducts = () => router.push({ path: '/standards/products' })
const goUpdates = () => router.push({ path: '/centers/updates' })
const goSubscription = () => router.push({ path: '/centers/subscription' })

onBeforeMount(() => {
  fetchData()
})
</script>

<style lang="scss">
.purchased-box {
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

  .type-filter {
    margin-bottom: 12px;
  }

  .empty-actions {
    display: flex;
    justify-content: center;
    padding: 16px 0;
  }
}
</style>
