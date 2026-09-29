<!--
 * @Author: ${git_name}
 * @Date: 2025-04-21 15:24:09
 * @LastEditors: ${git_name}
 * @LastEditTime: 2026-09-11 10:00:00
 * @FilePath: /books/web/src/views/centers/credits/index.vue
 * @Description: 积分中心：余额 / 充值套餐 / 消费流水 / 兑换码
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="listLoading" class="list-container table-auto-height credits-box">
    <!-- 余额总览 -->
    <el-card class="balance-card" shadow="never">
      <div class="balance-row">
        <div class="balance-item">
          <div class="balance-label">当前积分余额</div>
          <div class="balance-value">{{ balance }}</div>
        </div>
        <div class="balance-item">
          <div class="balance-label">累计充值</div>
          <div class="balance-value sub">{{ totalRecharge }}</div>
        </div>
        <div class="balance-item">
          <div class="balance-label">累计消费</div>
          <div class="balance-value sub">{{ totalConsume }}</div>
        </div>
        <div class="balance-actions">
          <el-button :icon="Wallet" type="primary" @click="openRecharge">充值积分</el-button>
          <el-button :icon="Ticket" @click="redeemVisible = true">兑换码</el-button>
        </div>
      </div>
      <div class="balance-tip">
        产品标准支持 5 积分单独购买（购买后保留购买时版本，不随标准更新同步）；材料 / 性能 / 表面标准免费开放；
        订阅全年可获得全部产品标准及当年标准更新。
      </div>
    </el-card>

    <el-tabs v-model="TabsActiveName" class="credits-tabs" @tab-change="changeDocsTab">
      <template v-for="(item, index) in docsTabs" :key="'credits' + index">
        <el-tab-pane :name="item.value">
          <template #label>
            <div class="tabs-label">
              <template v-if="index == 0">
                <el-icon size="20">
                  <operation />
                </el-icon>
              </template>
              <template v-if="index == 1">
                <el-icon size="20">
                  <folder-checked />
                </el-icon>
              </template>
              <template v-if="index == 2">
                <el-icon size="20">
                  <folder-delete />
                </el-icon>
              </template>
              <el-text>{{ item.label }}</el-text>
            </div>
          </template>
          <el-table v-loading="listLoading" :border="true" :data="rows.list" :stripe="true" style="width: 100%">
            <el-table-column align="center" label="编号" prop="id" width="80" />
            <el-table-column align="center" label="类型" prop="source" width="120">
              <template #default="{ row }">
                <el-tag :type="Number(row.delta) >= 0 ? 'success' : 'warning'">
                  {{ Number(row.delta) >= 0 ? '获取' : '消耗' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column align="center" label="来源" prop="source" width="120" />
            <el-table-column align="center" label="积分变动" width="120">
              <template #default="{ row }">
                <span :class="Number(row.delta) >= 0 ? 'delta-up' : 'delta-down'">
                  {{ Number(row.delta) >= 0 ? `+${row.delta}` : row.delta }}
                </span>
              </template>
            </el-table-column>
            <el-table-column align="center" label="变动后余额" prop="balance_after" width="120" />
            <el-table-column align="center" label="备注" prop="remark" show-overflow-tooltip>
              <template #default="{ row }">
                {{ remarkText(row) }}
              </template>
            </el-table-column>
            <el-table-column align="center" label="时间" prop="created_at" width="180" />
            <template #empty>
              <el-empty class="vab-data-empty" description="暂无数据" />
            </template>
          </el-table>
          <el-pagination
            v-if="rows.total > 0"
            background
            :current-page="queryForm.page"
            :layout="layout"
            :page-size="queryForm.pageSize"
            :total="rows.total"
            @current-change="handleCurrentChange"
          />
        </el-tab-pane>
      </template>
    </el-tabs>

    <!-- 充值套餐 -->
    <el-dialog v-model="rechargeVisible" :close-on-click-modal="false" title="充值积分" width="560px">
      <div v-loading="packageLoading" class="package-list">
        <div
          v-for="pkg in packages"
          :key="pkg.id"
          class="package-item"
          :class="{ active: Number(selectedPackageId) === Number(pkg.id) }"
          @click="selectedPackageId = Number(pkg.id)"
        >
          <div class="package-credits">{{ pkg.credits }} 积分</div>
          <div class="package-price">￥{{ (Number(pkg.price || 0) / 100).toFixed(2) }}</div>
          <div v-if="pkg.tag" class="package-tag">{{ pkg.tag }}</div>
        </div>
      </div>
      <div class="channel-row">
        <span class="channel-label">支付方式</span>
        <el-radio-group v-model="channel">
          <el-radio :label="1">微信支付</el-radio>
          <el-radio :label="2">支付宝</el-radio>
        </el-radio-group>
      </div>
      <el-alert
        v-if="orderInfo"
        class="order-alert"
        :closable="false"
        show-icon
        :title="`订单号 ${orderInfo.order_no || '—'}，金额 ￥${(Number(orderInfo.amount || 0) / 100).toFixed(2)}，请使用${Number(channel) === 1 ? '微信' : '支付宝'}扫码完成支付。如二维码未弹出，请联系客服 13216118255。`"
        type="success"
      />
      <template #footer>
        <el-button @click="rechargeVisible = false">关闭</el-button>
        <el-button :loading="rechargeLoading" type="primary" @click="submitRecharge">确认充值</el-button>
      </template>
    </el-dialog>

    <!-- 兑换码 -->
    <el-dialog v-model="redeemVisible" :close-on-click-modal="false" title="兑换码兑换" width="420px">
      <el-input v-model="redeemCode" clearable placeholder="请输入积分兑换码" />
      <template #footer>
        <el-button @click="redeemVisible = false">取消</el-button>
        <el-button :loading="redeemLoading" type="primary" @click="submitRedeem">确认兑换</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts" setup>
import { Operation, FolderDelete, FolderChecked, Wallet, Ticket } from '@element-plus/icons-vue'
import { creditsBalance, creditsPackages, creditsPurchase, creditsTransactions, creditsRedeem } from '/@/api/credits'
import { getStorage } from '/@/utils/storage'
import { useUserStore } from '/@/store/modules/user'
const userStore = useUserStore()
const { credit } = storeToRefs(userStore)
const $baseMessage = inject<any>('$baseMessage')

const templateName = 'CentersCredits'
defineOptions({
  name: templateName,
})

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
const listLoading = ref<boolean>(false)
const rows = reactive({
  total: 0,
  list: [] as any[],
})

/* 余额总览（服务端为唯一可信源，同时回写 user store） */
const balance = ref<number>(Number(credit?.value ?? 0))
const totalRecharge = ref<number>(0)
const totalConsume = ref<number>(0)

const TabsActiveName = ref('all')
const docsTabs = reactive([
  {
    label: '所有记录',
    value: 'all',
  },
  {
    label: '获取记录',
    value: 'obtain',
  },
  {
    label: '消耗记录',
    value: 'consume',
  },
])
const layout = ref('prev, pager, next, total')

const remarkText = (row: any) => {
  const remark = row?.remark
  if (!remark) return '—'
  if (typeof remark === 'string') return remark
  return remark.title || remark.type || remark.ref_type || '—'
}

const fetchBalance = async () => {
  try {
    const { data }: any = await creditsBalance()
    balance.value = Number(data?.balance ?? 0)
    totalRecharge.value = Number(data?.total_recharge ?? 0)
    totalConsume.value = Number(data?.total_consume ?? 0)
    userStore.setCredit(balance.value)
  } catch (error) {
    console.error('[centersCredits] 积分余额加载失败:', error)
  }
}

const fetchTransactions = async () => {
  listLoading.value = true
  try {
    const { data }: any = await creditsTransactions({ page: queryForm.page, pageSize: queryForm.pageSize })
    const list = Array.isArray(data?.list) ? data.list : []
    const filtered = list.filter((item: any) => {
      if (TabsActiveName.value === 'obtain') return Number(item?.delta ?? 0) >= 0
      if (TabsActiveName.value === 'consume') return Number(item?.delta ?? 0) < 0
      return true
    })
    rows.list = filtered
    rows.total = TabsActiveName.value === 'all' ? Number(data?.total ?? filtered.length) : filtered.length
  } catch (error) {
    console.error('[centersCredits] 积分流水加载失败:', error)
    rows.list = []
    rows.total = 0
  } finally {
    listLoading.value = false
  }
}

const changeDocsTab = () => {
  queryForm.page = 1
  fetchTransactions()
}

const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchTransactions()
}

/* ─── 充值 ─── */
const rechargeVisible = ref(false)
const packageLoading = ref(false)
const rechargeLoading = ref(false)
const packages = ref<any[]>([])
const selectedPackageId = ref<number>(0)
const channel = ref<number>(1)
const orderInfo = ref<any>(null)

const openRecharge = async () => {
  rechargeVisible.value = true
  orderInfo.value = null
  if (packages.value.length) return
  packageLoading.value = true
  try {
    const { data }: any = await creditsPackages()
    packages.value = Array.isArray(data?.list) ? data.list : []
    if (packages.value.length) selectedPackageId.value = Number(packages.value[0]?.id ?? 0)
  } catch (error) {
    console.error('[centersCredits] 积分套餐加载失败:', error)
    $baseMessage?.('积分套餐加载失败，请稍后重试', 'warning', 'hey')
  } finally {
    packageLoading.value = false
  }
}

const submitRecharge = async () => {
  if (!selectedPackageId.value) {
    $baseMessage?.('请先选择充值套餐', 'warning', 'hey')
    return
  }
  rechargeLoading.value = true
  try {
    const { data }: any = await creditsPurchase({ package_id: selectedPackageId.value, channel: channel.value })
    orderInfo.value = data ?? {}
    $baseMessage?.('充值订单已创建，请完成支付', 'success', 'hey')
  } catch (error) {
    console.error('[centersCredits] 充值下单失败:', error)
  } finally {
    rechargeLoading.value = false
  }
}

/* ─── 兑换码 ─── */
const redeemVisible = ref(false)
const redeemLoading = ref(false)
const redeemCode = ref('')

const submitRedeem = async () => {
  const code = String(redeemCode.value || '').trim()
  if (!code) {
    $baseMessage?.('请输入兑换码', 'warning', 'hey')
    return
  }
  redeemLoading.value = true
  try {
    const { data }: any = await creditsRedeem({ code })
    if (typeof data?.balance === 'number') {
      balance.value = Number(data.balance)
      userStore.setCredit(balance.value)
    }
    $baseMessage?.(`兑换成功，获得 ${data?.credits ?? 0} 积分`, 'success', 'hey')
    redeemCode.value = ''
    redeemVisible.value = false
    await fetchTransactions()
  } catch (error) {
    console.error('[centersCredits] 兑换码兑换失败:', error)
  } finally {
    redeemLoading.value = false
  }
}

onBeforeMount(() => {
  listLoading.value = true
  Promise.all([fetchBalance(), fetchTransactions()]).finally(() => {
    listLoading.value = false
  })
})
</script>

<style lang="scss">
.credits-box {
  background: rgb(255, 255, 255);

  .balance-card {
    margin-bottom: 12px;

    .balance-row {
      display: flex;
      flex-wrap: wrap;
      gap: 48px;
      align-items: center;
    }

    .balance-label {
      font-size: 13px;
      color: #909399;
    }

    .balance-value {
      font-size: 26px;
      font-weight: bold;
      color: #052965;
    }

    .balance-value.sub {
      font-size: 18px;
      color: #606266;
    }

    .balance-actions {
      margin-left: auto;
    }

    .balance-tip {
      margin-top: 10px;
      font-size: 12px;
      line-height: 18px;
      color: #909399;
    }
  }

  .delta-up {
    font-weight: bold;
    color: #67c23a;
  }

  .delta-down {
    font-weight: bold;
    color: #e6a23c;
  }

  .package-list {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    min-height: 90px;
  }

  .package-item {
    width: 150px;
    padding: 12px;
    text-align: center;
    cursor: pointer;
    border: 1px solid #dcdfe6;
    border-radius: 6px;

    &.active {
      background: #eaf0f0;
      border-color: #052965;
    }

    .package-credits {
      font-size: 16px;
      font-weight: bold;
      color: #052965;
    }

    .package-price {
      margin-top: 6px;
      font-size: 14px;
      color: #e6a23c;
    }

    .package-tag {
      margin-top: 4px;
      font-size: 12px;
      color: #909399;
    }
  }

  .channel-row {
    margin-top: 16px;

    .channel-label {
      margin-right: 16px;
      color: #606266;
    }
  }

  .order-alert {
    margin-top: 12px;
  }

  .tabs-label {
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;

    img {
      width: 18px;
      height: auto;
      margin-right: 6px;
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
