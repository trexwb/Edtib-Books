<!--
 * @Author: trexwb
 * @Date: 2025-04-12 09:46:20
 * @LastEditors: trexwb
 * @LastEditTime: 2025-10-11 14:13:57
 * @FilePath: /client/books/web/src/views/standards/docs/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="comprehensive-table-container table-auto-height">
    <el-alert
      class="docs-alert"
      :closable="false"
      description="资料文档下载需消耗对应积分，同一文档不重复扣费；下载记录可在「个人中心」查看。"
      show-icon
      title="资料文档为积分下载内容"
      type="info"
    />
    <vab-query-form>
      <vab-query-form-top-panel>
        <el-form :inline="true" label-width="80px" :model="queryForm" @submit.prevent>
          <el-form-item label="模糊搜索">
            <el-input v-model="queryForm.filter.keywords" clearable placeholder="请输入关键字模糊搜索" />
          </el-form-item>
          <el-form-item>
            <el-button :icon="Search" :loading="state.loading" native-type="submit" type="primary" @click="queryData">查询</el-button>
          </el-form-item>
        </el-form>
      </vab-query-form-top-panel>
    </vab-query-form>
    <el-table ref="tableSortRef" v-loading="state.loading" :border="true" :data="rows.list" :stripe="true">
      <el-table-column
        v-for="(item, index) in finallyColumns"
        :key="index"
        align="center"
        :label="item.label"
        :prop="item.prop"
        show-overflow-tooltip
        :width="item.width"
      >
        <template #default="{ row }">
          <template v-if="item.prop === 'title'">
            <div style="text-align: left">
              <el-link type="primary" @click="handleView(row)">
                {{ row.title.substring(0, row.title.lastIndexOf('.')) }}
              </el-link>
            </div>
          </template>
        </template>
      </el-table-column>
      <el-table-column align="center" fixed="right" label="操作" width="140">
        <template #default="{ row }">
          <el-button :icon="Download" link :loading="downloadingId === Number(row.id)" type="primary" @click="handleDownload(row)">
            {{ Number(row.credit || 0) > 0 ? `${row.credit} 积分下载` : '下载' }}
          </el-button>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty class="vab-data-empty" description="暂无数据" />
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
    <products-docs ref="docsRef" />
  </div>
</template>

<script lang="ts" setup>
import { useAclStore } from '/@/store/modules/acl'
import { Delete, Download, Plus, Search, Upload, Edit, View } from '@element-plus/icons-vue'
import { downloadsLogs, docsHits } from '/@/api/download'
import { docsList, docsDownload } from '/@/api/docs'
import { getStorage, setStorage } from '/@/utils/storage'
import { ElMessageBox } from 'element-plus'
import { useUserStore } from '/@/store/modules/user'

const userStore = useUserStore()
const $baseMessage = inject<any>('$baseMessage') // 注入全局消息提示方法

const templateName = 'WikisDocsFiles'
defineOptions({
  name: templateName,
})

const layout = ref('sizes, prev, pager, next, jumper, total')
const docsRef = ref<any>(null)
const viewRef = ref<any>(null)

const state = reactive({
  operationFixed: true,
  selectionRows: [],
  fold: true,
  loading: false,
})

const queryForm = reactive({
  filter: {
    keywords: '',
    id: '',
    title: '',
    status: '',
  },
  page: 1,
  pageSize: Number(getStorage('pageSize') || 20),
  sort: '+id',
})

const rows = reactive({
  total: 0,
  list: [],
})

const columns = ref([
  {
    label: '文件名',
    prop: 'title',
    sortable: true,
    disableCheck: true,
  },
  {
    label: '所需积分',
    prop: 'credit',
    width: 120,
    sortable: true,
    disableCheck: false,
  },
  {
    label: '最后更新时间',
    prop: 'updated_at',
    width: 180,
    sortable: true,
    disableCheck: false,
  },
])
const myCheckList = getStorage('checkList') || {}
const checkList = ref<any>(myCheckList[templateName] || ['id', 'title', 'credit', 'times_expire', 'created_at', 'status'])
const finallyColumns = computed(() => {
  return columns.value.filter((item: any) => checkList.value.includes(item.prop))
})

const handleView = (row: any = {}) => {
  docsRef.value.showDetail(Number(row.id || 0))
}

/* ─── 资料文档：积分下载（对齐后端 FastenerCreditsService 钱包扣费口径） ─── */
const downloadingId = ref<number>(0)

const handleDownload = async (row: any = {}) => {
  const docId = Number(row.id || 0)
  if (!docId) return
  const credit = Number(row.credit || 0)
  if (credit > 0) {
    try {
      await ElMessageBox.confirm(`该资料文档需要 ${credit} 积分，确认下载？已下载过的文档不重复扣费。`, '积分下载', {
        confirmButtonText: '确认下载',
        cancelButtonText: '暂不下载',
        type: 'warning',
        draggable: false,
      })
    } catch {
      return // 用户取消
    }
  }
  downloadingId.value = docId
  try {
    const { data }: any = await docsDownload({ id: docId })
    const url = data?.url
    if (!url) {
      $baseMessage?.('下载地址获取失败，请稍后重试', 'warning', 'hey')
      return
    }
    // 余额扣减后由后端回写，前端同步本地钱包展示
    if (typeof data?.balance === 'number') userStore.setCredit(data.balance)
    const link = document.createElement('a')
    link.setAttribute('href', url)
    link.setAttribute('download', String(row.title || `doc-${docId}`))
    link.setAttribute('target', '_blank')
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (error: any) {
    console.error('[standardsDocs] 资料文档下载失败:', error)
    // 400400010002：积分余额不足，引导前往积分中心充值
    const code = Number(error?.code ?? error?.data?.code ?? 0)
    if (code === 400400010002) {
      $baseMessage?.('积分余额不足，请前往「积分中心」充值，或联系客服 13216118255', 'warning', 'hey')
    }
  } finally {
    downloadingId.value = 0
  }
}

const fetchData = async () => {
  try {
    if (state.loading) return
    state.loading = true
    const { data }: any = await docsList(queryForm)
    rows.list = data?.list || []
    rows.total = data?.total || 0
    state.loading = false
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  }
}

const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchData()
}

const handleSizeChange = (value: number) => {
  queryForm.page = 1
  queryForm.pageSize = value
  setStorage('pageSize', queryForm.pageSize)
  fetchData()
}
const queryData = () => {
  queryForm.page = 1
  fetchData()
}

const init = () => {
  fetchData()
}

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {
  init()
})
</script>

<style lang="scss" scoped>
.is-text {
  height: 12px !important;
  padding: 0 !important;
}
  .docs-alert {
    margin-bottom: 12px;
  }

</style>
