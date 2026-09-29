<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/standards/capabilities/index.vue
 * @Description: 性能标准（免费内容）：注册用户全部可见，无积分购买 / 订阅门禁。
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->
<template>
  <div class="list-container table-auto-height capabilities-container">
    <el-alert
      class="free-alert"
      :closable="false"
      description="无需积分购买或订阅即可查看"
      show-icon
      title="性能标准为免费内容，注册用户全部可见"
      type="success"
    />

    <div class="filter-row">
      <el-input
        v-model="queryForm.keyword"
        clearable
        placeholder="名称 / 编号"
        style="width: 260px"
        @clear="handleSearch"
        @keyup.enter="handleSearch"
      />
      <el-button :icon="Search" type="primary" @click="handleSearch">查询</el-button>
      <el-button :icon="Refresh" @click="handleReset">重置</el-button>
    </div>

    <el-table v-loading="listLoading" :border="true" :data="rows.list" :stripe="true" style="width: 100%">
      <el-table-column align="center" label="编号" prop="id" width="90" />
      <el-table-column align="center" label="名称" show-overflow-tooltip>
        <template #default="{ row }">{{ nameText(row) }}</template>
      </el-table-column>
      <el-table-column align="center" label="编号 / 代号" show-overflow-tooltip>
        <template #default="{ row }">{{ row.code || row.serial || row.number || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="说明" show-overflow-tooltip>
        <template #default="{ row }">{{ row.detail || row.remark || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="访问" width="90">
        <template #default>
          <el-tag effect="dark" size="small" type="success">免费</el-tag>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty class="vab-data-empty" :description="emptyText" />
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
    />
  </div>
</template>

<script lang="ts" setup>
import { Refresh, Search } from '@element-plus/icons-vue'
import { capabilitiesList } from '/@/api/capabilities'
import { getStorage } from '/@/utils/storage'

const templateName = 'StandardsCapabilities'
defineOptions({
  name: templateName,
})

/** 多语言 JSON 文本取值（names/titles 为 { zh_cn: '...' } 结构） */
const jsonText = (value: any): string => {
  if (!value) return ''
  if (typeof value === 'string') return value
  return String(value.zh_cn || value.zh || Object.values(value)[0] || '')
}
const nameText = (row: any): string => jsonText(row?.names) || jsonText(row?.title) || String(row?.name ?? '—')

const listLoading = ref(false)
const layout = ref('prev, pager, next, total')
const available = ref(true)
const emptyText = ref('暂无性能标准数据')
const queryForm = reactive({
  keyword: '',
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
})
const rows = reactive({
  total: 0,
  list: [] as any[],
})

const fetchData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await capabilitiesList({
      filter: { keyword: queryForm.keyword || undefined },
      page: queryForm.page,
      pageSize: queryForm.pageSize,
    })
    rows.list = Array.isArray(data?.list) ? data.list : []
    rows.total = Number(data?.total ?? rows.list.length)
    available.value = data?.available !== false
    emptyText.value = String(data?.message || (available.value ? '暂无性能标准数据' : '免费内容，数据接入中'))
  } catch (error) {
    console.error('[standardsCapabilities] 性能标准加载失败:', error)
    rows.list = []
    rows.total = 0
  } finally {
    listLoading.value = false
  }
}

const handleSearch = () => {
  queryForm.page = 1
  fetchData()
}
const handleReset = () => {
  queryForm.keyword = ''
  handleSearch()
}
const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchData()
}

onBeforeMount(() => {
  fetchData()
})
</script>

<style lang="scss">
.capabilities-container {
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
}
</style>
