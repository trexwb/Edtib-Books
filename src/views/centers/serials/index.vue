<!--
 * @Author: ${git_name}
 * @Date: 2025-04-21 15:24:09
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-13 17:20:07
 * @FilePath: /books/web/src/views/centers/serials/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="listLoading" class="list-container table-auto-height serials-box">
    <vab-query-form>
      <vab-query-form-top-panel>
        <el-form inline label-width="75px" :model="queryForm" @submit.prevent>
          <el-form-item label="模糊搜索" style="width: 340px">
            <el-input v-model="queryForm.filter.keywords" clearable placeholder="请输入关键字模糊搜索" style="width: 100%" />
          </el-form-item>
          <el-form-item v-show="!fold" label="序列号" style="width: 340px">
            <el-input v-model="queryForm.filter.code" clearable placeholder="请输入序列号" style="width: 100%" />
          </el-form-item>
          <el-form-item v-show="!fold" label="类型">
            <el-select v-model="queryForm.filter.type" clearable placeholder="请选择类型" style="width: 175px">
              <el-option label="全部" value="" />
              <el-option v-for="item in typeList" :key="item.value" :label="item.label" :value="item.value" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button :icon="Search" :loading="listLoading" native-type="submit" type="primary" @click="queryData">查询</el-button>
            <el-button class="hidden-xs-only" text type="primary" @click="handleFold">
              <span v-if="fold">展开</span>
              <span v-else>合并</span>
              <vab-icon class="vab-dropdown" :class="{ 'vab-dropdown-active': fold }" icon="arrow-up-s-line" />
            </el-button>
          </el-form-item>
        </el-form>
      </vab-query-form-top-panel>
    </vab-query-form>
    <div>
      <el-button :icon="Switch" style="position: relative; z-index: 500; float: right" type="primary" @click="exchange">
        点兑换优惠
      </el-button>
    </div>
    <el-tabs v-model="TabsActiveName" class="docs-tabs" style="clear: both; overflow: hidden" @tab-change="changeDocsTab">
      <template v-for="(item, index) in docsTabs" :key="'docs' + index">
        <el-tab-pane :name="item.value">
          <template #label>
            <div class="tabs-label">
              <template v-if="index == 0">
                <el-icon><select /></el-icon>
              </template>
              <template v-if="index == 1">
                <el-icon>
                  <close-bold />
                </el-icon>
              </template>
              <el-text>{{ item.label }}</el-text>
            </div>
          </template>
          <template v-if="TabsActiveName == 'use'">
            <el-empty v-if="useRow.total == 0" class="vab-data-empty el-table" description="暂无数据" />
          </template>
          <template v-if="TabsActiveName == 'used'">
            <el-table v-show="usedRows.total > 0" :border="true" :data="usedRows.list" :stripe="true" style="width: 100%">
              <el-table-column
                v-for="(item, index) in usedFinallyColumns"
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
                  <template v-if="item.prop === 'type'">{{ usedTypeList[row.type] }}</template>
                  <template v-if="item.prop === 'status'">
                    <el-tag type="info">{{ usedStatusList[row.status] }}</el-tag>
                  </template>
                  <template v-if="item.prop === 'code'">
                    <div v-html="highlightText(row.code, 'code')"></div>
                  </template>
                </template>
              </el-table-column>
            </el-table>
            <el-pagination
              v-if="!emptyShow"
              background
              :current-page="queryForm.page"
              :layout="layout"
              :page-size="queryForm.pageSize"
              :total="usedRows.total"
              @current-change="handleCurrentChange"
            />
            <el-empty v-if="emptyShow" class="vab-data-empty el-table" description="暂无数据" />
          </template>
        </el-tab-pane>
      </template>
    </el-tabs>

    <el-dialog v-model="dialogVisible" :before-close="handleClose" title="使用序列号兑换优惠" width="650">
      <div class="dialog-tip-content">请输入您的序列号进行优惠券兑换，序列号兑换成功后直接会将您的优惠券对应的续期或积分充值到您的账户</div>
      <el-form style="margin-right: 0">
        <el-form-item style="margin-left: 15px">
          <template v-for="(part, index) in serialArr" :key="index">
            <el-input
              :ref="setInputRef(index)"
              v-model="serialArr[index].value"
              class="input-part"
              clearable
              :disabled="part.disabled"
              maxlength="5"
              style="width: 92px"
              @blur="handleBlur(index)"
              @input="handleInput(index)"
              @paste="handlePaste($event, index)"
            />
            <span v-if="index < 4">&nbsp;&nbsp;—&nbsp;&nbsp;</span>
          </template>
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="cancel">取消</el-button>
          <el-button v-debounce="confirm" type="primary">确定</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts" setup>
import { useUserStore } from '/@/store/modules/user'
import { Select, CloseBold, Switch, Search } from '@element-plus/icons-vue'
import { serialsUse, serialsExchange } from '/@/api/serials'
import { getStorage } from '/@/utils/storage'
const TabsActiveName = ref('used')
const templateName = 'CentersSerials'
const $baseMessage = inject<any>('$baseMessage') // 注入全局消息提示方法

defineOptions({
  name: templateName,
})

const userStore = useUserStore()
const { credit } = storeToRefs(userStore)

/* 核心响应式状态 */
const state = reactive<any>({
  submit: false, // 表单提交状态
})

const fold = ref<boolean>(true)
const handleFold = () => {
  fold.value = !fold.value
}
// 定义 target 的合法类型
type FilterKeys = keyof typeof queryForm.filter
// HTML 转义：防止命中内容经 v-html 渲染时产生 XSS
const escapeHtml = (s: string): string =>
  s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] as string)
// 正则元字符转义：防止搜索词含 ( [ \ 等导致 new RegExp 抛 SyntaxError
const escapeRegExp = (s: string): string => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
// 修改 highlightText 方法，确保 target 类型安全
const highlightText = (text: any, target: FilterKeys) => {
  // 检查 target 是否为合法键
  if (!(target in queryForm.filter)) {
    // C-B6: 安全返回路径也必须转义，否则原始数据可注入 HTML（v-html 渲染出口）
    return escapeHtml(String(text))
  }
  const keyword = queryForm.filter[target]
  if (keyword === '' || keyword == null) {
    return escapeHtml(String(text))
  }
  const escapedText = escapeHtml(String(text))
  const safeKeyword = escapeRegExp(String(keyword))
  return escapedText.replace(new RegExp(safeKeyword, 'g'), (matched) => `<span style="color:#F56C6C">${matched}</span>`)
}
const queryData = () => {
  queryForm.page = 1
  if (TabsActiveName.value == 'used') {
    fetchUsedData()
  } else {
  }
}
const queryForm = reactive({
  filter: {
    keywords: '',
    code: '',
    type: '',
  },
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
  sort: '-id',
})
const inputRefs = ref<Record<string, HTMLInputElement>>({})
const setInputRef = (index: any) => {
  return (el: Element | ComponentPublicInstance | null) => {
    const inputEl = el as HTMLInputElement | null
    if (inputEl) {
      inputRefs.value[index] = inputEl
    } else {
      delete inputRefs.value[index]
    }
  }
}
// 定义序列号数组
const serialArr = ref([
  {
    value: '',
    disabled: false,
  },
  {
    value: '',
    disabled: true,
  },
  {
    value: '',
    disabled: true,
  },
  {
    value: '',
    disabled: true,
  },
  {
    value: '',
    disabled: true,
  },
])
// 计算完整序列号
const code = computed(() => {
  return serialArr.value.map((part) => part.value).join('-')
})
// 上一个输入框填写完毕后， 下一个输入解禁，获取焦点
const handleBlur = (index: any) => {
  if (serialArr.value[index].value) {
    if (index < serialArr.value.length - 1) {
      serialArr.value[index + 1].disabled = false
      nextTick(() => {
        if (serialArr.value[index].value && !serialArr.value[index + 1].value) {
          inputRefs.value[index + 1]?.focus()
        }
      })
    }
  }
}
// 取消
const cancel = () => {
  serialArr.value.map((item) => {
    item.value = ''
    item.disabled = true
  })
  serialArr.value[0].disabled = false
  dialogVisible.value = false
}
// 确定
const confirm = () => {
  if (!code.value) {
    $baseMessage('序列号不能为空', 'warning')
    return
  }
  serialsExchangeCoupons()
}
// 兑换优惠
const serialsExchangeCoupons = async () => {
  if (state.submit) return
  state.submit = true
  try {
    await serialsExchange({ code: code.value })
    $baseMessage('兑换成功', 'success')
    dialogVisible.value = false
    serialArr.value.map((item) => {
      item.value = ''
      item.disabled = true
    })
    serialArr.value[0].disabled = false
    // fetchUsedData()
    window.location.reload()
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  } finally {
    state.submit = true
  }
}
// 粘贴
const handlePaste = (event: any, index: any) => {
  event.preventDefault()
  const pastedText = event.clipboardData.getData('text')
  const newArr = pastedText.split('-')
  const newArrLen = newArr.length
  if (Number(index) === 0) {
    if (newArrLen > 0) {
      if (newArrLen == 5) {
        serialArr.value.map((item, itemIndex) => {
          item.value = newArr[itemIndex]
          item.disabled = false
        })
      } else {
        serialArr.value[index].value = newArr[0]
        serialArr.value[index].disabled = false
      }
    }
  } else {
    if (newArrLen > 0) {
      serialArr.value[index].value = newArr[0]
      serialArr.value[index].disabled = false
    }
  }
}
// 输入框变化
const handleInput = (index: any) => {
  // 如果当前输入框已填满 5 个字符，跳转到下一个输入框
  if (serialArr.value[index].value.length === 5 && index < serialArr.value.length - 1) {
    serialArr.value[index].value = serialArr.value[index].value.slice(0, 5)
  }
}
// 兑换优惠券弹层
const dialogVisible = ref(false)
const handleClose = () => {
  dialogVisible.value = false
}
const exchange = () => {
  dialogVisible.value = !dialogVisible.value
}
const emptyShow = ref<boolean>(false)
const listLoading = ref<boolean>(false)
const docsTabs = reactive([
  {
    label: '可用优惠券',
    value: 'use',
  },
  {
    label: '已用优惠券',
    value: 'used',
  },
])
// 类型
const usedTypeList = reactive(['续期', '积分', '升级', '设备', '综合'])

const typeList = reactive([
  {
    label: '续期',
    value: 0,
  },
  {
    label: '积分',
    value: 1,
  },
  {
    label: '升级',
    value: 2,
  },
  {
    label: '设备',
    value: 3,
  },
  {
    label: '综合',
    value: 4,
  },
])
// 状态
const usedStatusList = reactive(['待售不可用', '已售正常可用', '已用不可用', '回收禁用'])
const usedRows = reactive({
  total: 0,
  list: [],
})
const useRow = reactive({
  total: 0,
  list: [],
})
const usedColumns = ref([
  {
    label: '编号',
    prop: 'id',
    width: 80,
    sortable: true,
    fixed: true,
    disableCheck: false,
  },
  {
    label: '批次',
    prop: 'batch',
    width: 180,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '序列号',
    prop: 'code',
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '积分',
    prop: 'credit',
    width: 80,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '售价(元)',
    prop: 'price',
    width: 80,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    // 类型：0续期，1积分，2升级，3设备，4综合
    label: '类型',
    prop: 'type',
    width: 80,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '升级',
    prop: 'level',
    width: 80,
    sortable: false,
    fixed: false,
    disableCheck: false,
  },
  {
    label: '续期',
    prop: 'days',
    width: 180,
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
    // 状态：0待售不可用,1已售正常可用,2已用不可用,3回收禁用
    label: '状态',
    prop: 'status',
    width: 120,
    sortable: false,
    fixed: 'right',
    disableCheck: false,
  },
])
const myCheckList = getStorage('usedCheckList') || {}
const usedCheckList = ref<any>(
  myCheckList[templateName] || [
    'id',
    'batch',
    'code',
    'credit',
    'price',
    'type',
    'level',
    'days',
    'times_expire',
    'created_at',
    'updated_at',
    'status',
  ]
)
const usedFinallyColumns = computed(() => {
  return usedColumns.value.filter((item: any) => usedCheckList.value.includes(item.prop))
})

const layout = ref('prev, pager, next, total')
const changeDocsTab = () => {
  queryForm.page = 1
  if (TabsActiveName.value === 'use') {
    fetchUseData()
  } else {
    fetchUsedData()
  }
}

// 可用优惠券
const fetchUseData = async () => {
  listLoading.value = true
  setTimeout(() => {
    listLoading.value = false
  }, 450)
  // try {
  //   const { data }: any = await serialsUse(queryForm)
  //   listLoading.value = false
  //   emptyShow.value = data.total <= 0
  // } catch (error) {
  //   console.error('Fetch data error:', error)
  //   // 可以加入错误处理逻辑，如显示错误信息
  // }
}
// 已用优惠券
const fetchUsedData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await serialsUse(queryForm)
    usedRows.list = data?.list || []
    usedRows.total = data?.total || 0
    listLoading.value = false
    emptyShow.value = data.total <= 0
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  }
}
const handleCurrentChange = (value: number) => {
  queryForm.page = value
  // fetchUseData()
}
onBeforeMount(() => {
  fetchUsedData()
})
</script>

<style lang="scss">
.serials-box {
  width: 100%;
  background: rgb(255, 255, 255);

  .dialog-tip-content {
    margin: 10px 0 20px;
    font-size: 14px;
    line-height: 1.5;
  }

  .docs-tabs {
    margin-top: -20px;
  }

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
