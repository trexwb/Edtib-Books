<!--
 * @Author: ${git_name}
 * @Date: 2025-04-21 15:24:09
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-13 16:26:27
 * @FilePath: /books/web/src/views/centers/docs/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="list-container table-auto-height docs-box">
    <el-tabs v-model="TabsActiveName" class="docs-tabs" @tab-change="changeDocsTab">
      <template v-for="(item, index) in docsTabs" :key="'docs' + index">
        <el-tab-pane :name="item.value">
          <template #label>
            <div class="tabs-label">
              <template v-if="index == 0">
                <el-icon>
                  <download />
                </el-icon>
              </template>
              <template v-if="index == 1">
                <el-icon>
                  <view />
                </el-icon>
              </template>
              <template v-if="index == 2">
                <el-icon>
                  <tickets />
                </el-icon>
              </template>
              <el-text>{{ item.label }}</el-text>
            </div>
          </template>
          <template v-if="TabsActiveName == 'download'">
            <el-table v-loading="listLoading" :border="true" :data="downloadRows.list" :stripe="true" style="width: 100%">
              <el-table-column
                v-for="(item, index) in downloadFinallyColumns"
                :key="index"
                align="center"
                :fixed="item.fixed"
                :label="item.label"
                :prop="item.prop"
                show-overflow-tooltip
                :sortable="item.sortable"
                :width="item.width"
              >
                <template #default="{ row }">
                  <template v-if="item.prop === 'doc.title'">
                    <el-tooltip :content="row.doc.title" placement="top">
                      {{ row.doc.title }}
                    </el-tooltip>
                  </template>
                  <template v-if="item.prop === 'doc.credit'">{{ row.doc.credit }}</template>
                  <template v-if="item.prop === 'status'">
                    <el-tag :type="Number(row.status) === 1 ? 'primary' : 'info'">{{ Number(row.status) === 1 ? '启用' : '禁用' }}</el-tag>
                  </template>
                </template>
              </el-table-column>
              <el-table-column align="center" :fixed="downloadRows.operationFixed ? 'right' : false" label="操作" width="100">
                <template #header>
                  <el-checkbox v-model="downloadRows.operationFixed" label="固定" size="large" value="固定" />
                </template>
                <template #default="{ row }">
                  <el-tooltip content="预览" placement="bottom">
                    <el-icon size="20" @click="handleView(row)">
                      <view />
                    </el-icon>
                  </el-tooltip>
                  <el-tooltip content="下载" placement="bottom">
                    <el-icon size="20" @click="downloadPdf(row)">
                      <download />
                    </el-icon>
                  </el-tooltip>
                </template>
              </el-table-column>
            </el-table>
            <el-pagination
              v-if="!downloadEmptyShow"
              background
              :current-page="queryForm.page"
              :layout="layout"
              :page-size="queryForm.pageSize"
              :total="downloadRows.total"
              @current-change="handleCurrentChange"
            />
            <el-empty v-if="downloadEmptyShow" class="vab-data-empty el-table" description="暂无数据" />
          </template>
          <template v-if="TabsActiveName == 'knowledge'">
            <el-table v-loading="knowledgeLoading" :border="true" :data="knowledgeRows.list" :stripe="true" style="width: 100%">
              <el-table-column align="center" label="编号" prop="id" width="80" />
              <el-table-column align="center" label="文档标题" prop="title" show-overflow-tooltip />
              <el-table-column align="center" label="分类" prop="category" width="140" />
              <el-table-column align="center" label="所需积分" prop="price_credit" width="100" />
              <el-table-column align="center" label="状态" width="100">
                <template #default="{ row }">
                  <el-tag :type="row.owned ? 'success' : 'info'">{{ row.owned ? '已购' : '未购' }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column align="center" label="操作" width="160">
                <template #default="{ row }">
                  <el-button
                    v-if="!row.owned"
                    link
                    :loading="knowledgeBuyingId === Number(row.id)"
                    type="warning"
                    @click="purchaseKnowledge(row)"
                  >
                    积分购买
                  </el-button>
                  <el-button v-else link type="primary" @click="downloadKnowledge(row)">下载</el-button>
                </template>
              </el-table-column>
              <template #empty>
                <el-empty class="vab-data-empty" description="暂无数据" />
              </template>
            </el-table>
            <el-pagination
              v-if="knowledgeRows.total > queryForm.pageSize"
              background
              :current-page="queryForm.page"
              :layout="layout"
              :page-size="queryForm.pageSize"
              :total="knowledgeRows.total"
              @current-change="handleCurrentChange"
            />
          </template>
          <template v-if="TabsActiveName == 'owned'">
            <el-table v-loading="ownedLoading" :border="true" :data="ownedRows.list" :stripe="true" style="width: 100%">
              <el-table-column align="center" label="编号" prop="id" width="80" />
              <el-table-column align="center" label="文档标题" prop="title" show-overflow-tooltip />
              <el-table-column align="center" label="分类" prop="category" width="140" />
              <el-table-column align="center" label="所购积分" prop="price_credit" width="100" />
              <el-table-column align="center" label="购买时间" prop="purchased_at" width="180" />
              <el-table-column align="center" label="操作" width="120">
                <template #default="{ row }">
                  <el-button link type="primary" @click="downloadKnowledge(row)">下载</el-button>
                </template>
              </el-table-column>
              <template #empty>
                <el-empty class="vab-data-empty" description="暂无已购资料文档" />
              </template>
            </el-table>
            <el-pagination
              v-if="ownedRows.total > queryForm.pageSize"
              background
              :current-page="queryForm.page"
              :layout="layout"
              :page-size="queryForm.pageSize"
              :total="ownedRows.total"
              @current-change="handleCurrentChange"
            />
          </template>
          <template v-if="TabsActiveName == 'view'">
            <el-table v-loading="listLoading" :border="true" :data="viewRows.list" :stripe="true" style="width: 100%">
              <el-table-column
                v-for="(item, index) in viewFinallyColumns"
                :key="index"
                align="center"
                :fixed="item.fixed"
                :label="item.label"
                :prop="item.prop"
                show-overflow-tooltip
                :sortable="item.sortable"
                :width="item.width"
              >
                <template #default="{ row }">
                  <template v-if="item.prop === 'doc.title'">
                    <el-tooltip :content="row.doc.title" placement="top">
                      {{ row.doc.title }}
                    </el-tooltip>
                  </template>
                  <template v-if="item.prop === 'doc.credit'">{{ row.doc.credit }}</template>
                  <template v-if="item.prop === 'status'">
                    <el-tag :type="Number(row.status) === 1 ? 'primary' : 'info'">{{ Number(row.status) === 1 ? '启用' : '禁用' }}</el-tag>
                  </template>
                </template>
              </el-table-column>
              <el-table-column align="center" :fixed="downloadRows.operationFixed ? 'right' : false" label="操作" width="100">
                <template #header>
                  <el-checkbox v-model="downloadRows.operationFixed" label="固定" size="large" value="固定" />
                </template>
                <template #default="{ row }">
                  <el-tooltip content="下载" placement="bottom">
                    <el-button :icon="Download" text type="primary" @click="downloadPdf(row)" />
                  </el-tooltip>
                </template>
              </el-table-column>
            </el-table>
            <el-pagination
              v-if="!viewEmptyShow"
              background
              :current-page="queryForm.page"
              :layout="layout"
              :page-size="queryForm.pageSize"
              :total="total"
              @current-change="handleCurrentChange"
            />
            <el-empty v-if="viewEmptyShow" class="vab-data-empty el-table" description="暂无数据" />
          </template>
        </el-tab-pane>
      </template>
    </el-tabs>
    <products-docs ref="docsRef" />
  </div>
</template>

<script lang="ts" setup>
import { Download, Tickets, View } from '@element-plus/icons-vue'
import { downloadsLogs, docsHits } from '/@/api/download'
import { docsDownload } from '/@/api/products'
import { knowledgeList, knowledgeOwned, knowledgePurchase, knowledgeDownloadUrl } from '/@/api/knowledge'
import { useUserStore } from '/@/store/modules/user'
import { getStorage } from '/@/utils/storage'
import { ElMessageBox } from 'element-plus'
const $baseMessage = inject<any>('$baseMessage') // 注入全局消息提示方法
const userStore = useUserStore() // 积分余额同步

/** 多语言 JSON 文本取值（titles 为 { zh_cn: '...' } 结构） */
const jsonText = (value: any): string => {
  if (!value) return ''
  if (typeof value === 'string') return value
  return String(value.zh_cn || value.zh || Object.values(value)[0] || '')
}

const TabsActiveName = ref('download')
const templateName = 'CentersDocs'
defineOptions({
  name: templateName,
})

const downloadEmptyShow = ref<boolean>(false)
const listLoading = ref<boolean>(true)
const docsRef = ref<any>(null)

const list = ref<any>([])
const total = ref<any>(0)
const docsTabs = reactive([
  {
    label: '下载过的',
    value: 'download',
  },
  {
    label: '知识库文档',
    value: 'knowledge',
  },
  {
    label: '我的已购文档',
    value: 'owned',
  },
  // {
  //   label: '浏览过的',
  //   value: 'view',
  // },
])
const layout = ref('prev, pager, next, total')
const changeDocsTab = () => {
  listLoading.value = true
  queryForm.page = 1
  if (TabsActiveName.value === 'download') {
    fetchDownloadData()
  } else if (TabsActiveName.value === 'knowledge') {
    fetchKnowledgeData()
  } else if (TabsActiveName.value === 'owned') {
    fetchOwnedData()
  } else {
    fetchViewerData()
  }
}

const queryForm = reactive({
  filter: {
    keywords: '',
    id: '',
    name: '',
    product: '',
  },
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
  sort: '-id',
})
const downloadRows = reactive({
  total: 0,
  list: [],
  operationFixed: true,
})
const downloadColumns = ref([
  {
    label: '编号',
    prop: 'id',
    width: 80,
    sortable: true,
    fixed: true,
    disableCheck: false,
  },
  {
    label: '文件名',
    prop: 'doc.title',
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '下载积分',
    prop: 'doc.credit',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '下载次数',
    prop: 'use_limit',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '使用积分',
    prop: 'use_credit',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '有效期',
    prop: 'times_expire',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '创建时间',
    prop: 'created_at',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '最后更新时间',
    prop: 'updated_at',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '状态',
    prop: 'status',
    width: 120,
    sortable: false,
    disableCheck: false,
  },
])
const CheckList = getStorage('downloadCheckList') || {}
const downloadCheckList = ref<any>(
  CheckList[templateName] || ['id', 'doc.title', 'use_limit', 'use_credit', 'times_expire', 'created_at', 'updated_at']
)
const downloadFinallyColumns = computed(() => {
  return downloadColumns.value.filter((item: any) => downloadCheckList.value.includes(item.prop))
})
const fetchDownloadData = async () => {
  listLoading.value = true
  list.value = []
  total.value = 0
  try {
    const { data }: any = await downloadsLogs(queryForm)
    downloadRows.list = data?.list || []
    downloadRows.total = data?.total || 0
    listLoading.value = false
    downloadEmptyShow.value = data.total <= 0
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  }
}
// 下载
/* ─── 知识库文档：积分购买 + 下载（对齐后端 FastenerKnowledgeService） ─── */
const knowledgeLoading = ref<boolean>(false)
const knowledgeBuyingId = ref<number>(0)
const knowledgeRows = reactive({ total: 0, list: [] as any[] })

const fetchKnowledgeData = async () => {
  knowledgeLoading.value = true
  try {
    const { data }: any = await knowledgeList({
      keyword: String(queryForm.filter.keywords || ''),
      page: queryForm.page,
      pageSize: queryForm.pageSize,
    })
    const list = (Array.isArray(data?.list) ? data.list : []).map((item: any) => ({
      ...item,
      id: Number(item?.id ?? 0),
      title: jsonText(item?.titles) || String(item?.title ?? `文档 ${item?.id ?? ''}`),
      price_credit: Number(item?.priceCredit ?? item?.price_credit ?? 0),
      category: String(item?.category ?? ''),
      owned: !!item?.owned,
    }))
    knowledgeRows.list = list
    knowledgeRows.total = Number(data?.total ?? list.length)
  } catch (error) {
    console.error('[centersDocs] 知识库文档加载失败:', error)
    knowledgeRows.list = []
    knowledgeRows.total = 0
  } finally {
    knowledgeLoading.value = false
  }
}

const purchaseKnowledge = async (row: any) => {
  const itemId = Number(row?.id || 0)
  if (!itemId) return
  const price = Number(row?.price_credit || 0)
  try {
    await ElMessageBox.confirm(`是否使用 ${price} 积分购买该资料文档？购买后可随时下载。`, '积分购买', {
      confirmButtonText: '确认购买',
      cancelButtonText: '暂不购买',
      type: 'warning',
      draggable: false,
    })
  } catch {
    return // 用户取消
  }
  knowledgeBuyingId.value = itemId
  try {
    const { data }: any = await knowledgePurchase({ item_id: itemId })
    if (typeof data?.balance === 'number') userStore.setCredit(data.balance)
    $baseMessage?.('购买成功，可直接下载', 'success', 'hey')
    await fetchKnowledgeData()
  } catch (error: any) {
    console.error('[centersDocs] 资料文档购买失败:', error)
    // 400400010002：积分余额不足，引导前往积分中心充值
    if (Number(error?.code ?? 0) === 400400010002) {
      $baseMessage?.('积分余额不足，请前往「积分中心」充值，或联系客服 13216118255', 'warning', 'hey')
    }
  } finally {
    knowledgeBuyingId.value = 0
  }
}

const downloadKnowledge = async (row: any) => {
  const itemId = Number(row?.id || 0)
  if (!itemId) return
  try {
    const { data }: any = await knowledgeDownloadUrl({ item_id: itemId })
    const url = data?.url
    if (!url) {
      $baseMessage?.('下载地址获取失败，请稍后重试', 'warning', 'hey')
      return
    }
    // /storage 静态通道直接打开；签名占位地址同样走新窗口下载
    const link = document.createElement('a')
    link.setAttribute('href', url)
    link.setAttribute('download', data?.file_name || `knowledge-${itemId}`)
    link.setAttribute('target', '_blank')
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (error) {
    console.error('[centersDocs] 资料文档下载失败:', error)
  }
}

/* ─── 我的已购资料文档（对齐后端 /front/knowledge/owned） ─── */
const ownedLoading = ref<boolean>(false)
const ownedRows = reactive({ total: 0, list: [] as any[] })

const fetchOwnedData = async () => {
  ownedLoading.value = true
  try {
    const { data }: any = await knowledgeOwned()
    const list = (Array.isArray(data?.list) ? data.list : []).map((item: any) => ({
      ...item,
      id: Number(item?.id ?? item?.item_id ?? 0),
      title: jsonText(item?.titles) || String(item?.title ?? `文档 ${item?.id ?? ''}`),
      price_credit: Number(item?.priceCredit ?? item?.price_credit ?? 0),
      category: String(item?.category ?? ''),
      purchased_at: item?.purchased_at ?? item?.purchasedAt ?? '',
    }))
    ownedRows.list = list
    ownedRows.total = Number(data?.total ?? list.length)
  } catch (error) {
    console.error('[centersDocs] 已购资料文档加载失败:', error)
    ownedRows.list = []
    ownedRows.total = 0
  } finally {
    ownedLoading.value = false
  }
}

const downloadPdf = async (row: any) => {
  try {
    if (!row.doc_id) {
      ElMessageBox.alert('缺少必要的参数 ID，请重试!', '错误', {
        confirmButtonText: '确定',
      })
      return
    }
    const { data }: any = await docsDownload({ id: Number(row.doc_id) })
    // 封装为一个 Blob 对象， 生成一个临时的 URL。可以被浏览器用来访问文件内容，便于后续创建下载链接。
    const url = window.URL.createObjectURL(new Blob([data.url]))
    // 动态创建一个 HTML a 的元素，触发文件下载
    const link = document.createElement('a')
    // 指定 a 元素的href 属性，指定下载文件的来源。
    link.href = url
    // 设置 a 元素的 download 属性，指定下载文件的名称。
    link.setAttribute('download', row.doc.title) // 设置下载的文件名
    document.body.appendChild(link)
    link.click()
    link.remove()
    $baseMessage('下载成功', 'success')
  } catch (error) {
    console.error('There was an error downloading the file!', error)
  }
}

const viewEmptyShow = ref(false)
const viewRows = reactive({
  total: 0,
  list: [],
  operationFixed: true,
})
const viewColumns = ref([
  {
    label: '编号',
    prop: 'id',
    width: 80,
    sortable: true,
    fixed: true,
    disableCheck: false,
  },
  {
    label: '文件名',
    prop: 'doc',
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '下载积分',
    prop: 'doc',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '下载次数',
    prop: 'use_limit',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '使用积分',
    prop: 'use_credit',
    width: 100,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '有效期',
    prop: 'times_expire',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '创建时间',
    prop: 'created_at',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '最后更新时间',
    prop: 'updated_at',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '状态',
    prop: 'status',
    width: 120,
    sortable: false,
    disableCheck: false,
  },
])
const CheckLists = getStorage('downloadCheckList') || {}
const viewCheckList = ref<any>(
  CheckLists[templateName] || ['id', 'doc', 'doc', 'use_limit', 'use_credit', 'times_expire', 'created_at', 'updated_at', 'status']
)
const viewFinallyColumns = computed(() => {
  return viewColumns.value.filter((item: any) => viewCheckList.value.includes(item.prop))
})
const fetchViewerData = async () => {
  listLoading.value = true
  list.value = []
  total.value = 0
  try {
    const { data }: any = await docsHits(queryForm)
    viewRows.list = data?.list || []
    viewRows.total = data?.total || 0
    listLoading.value = false
    viewEmptyShow.value = data.total <= 0
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  }
}
const handleCurrentChange = (value: number) => {
  queryForm.page = value
  if (TabsActiveName.value === 'knowledge') {
    fetchKnowledgeData()
  } else if (TabsActiveName.value === 'owned') {
    fetchOwnedData()
  } else {
    fetchDownloadData()
  }
}
const handleView = (row: any) => {
  docsRef.value.showDetail(Number(row?.id || 0))
}
onBeforeMount(() => {
  fetchDownloadData()
})
</script>

<style lang="scss">
.docs-box {
  background: rgb(255, 255, 255);

  .tabs-label {
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;

    .el-icon {
      font-size: 20px;
    }
  }

  .el-tabs__item {
    width: 130px;
    padding: 0;
    text-align: center;

    &.is-active {
      background: #eaf0f0;

      .el-text {
        color: #052965;
      }
    }
  }

  .el-row {
    padding: 20px 0;
  }

  .name {
    font-size: 16px;
    font-weight: bold;
    color: #052965;
  }

  .operate {
    font-size: 16px;
    color: #052965;
  }
}
</style>
