<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/centers/orders/index.vue
 * @Description: 订单管理：积分充值 / 订阅订单列表，待支付订单可继续扫码支付或取消
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->

<template>
  <div class="list-container table-auto-height orders-box">
    <el-tabs v-model="TabsActiveName" class="orders-tabs" @tab-change="changeTab">
      <template v-for="(item, index) in statusTabs" :key="'orders' + index">
        <el-tab-pane :name="item.value">
          <template #label>
            <div class="tabs-label">
              <el-icon v-if="index == 0" size="20"><operation /></el-icon>
              <el-icon v-else-if="index == 1" size="20"><folder-checked /></el-icon>
              <el-icon v-else size="20"><folder-delete /></el-icon>
              <el-text>{{ item.label }}</el-text>
            </div>
          </template>
        </el-tab-pane>
      </template>
    </el-tabs>

    <el-table v-loading="listLoading" :border="true" :data="rows.list" :stripe="true" style="width: 100%">
      <el-table-column align="center" label="订单号" prop="order_no" width="220" />
      <el-table-column align="center" label="类型" width="100">
        <template #default="{ row }">
          <el-tag :type="Number(row.scene) === 1 ? 'success' : 'primary'">
            {{ sceneText(row.scene) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column align="center" label="内容" show-overflow-tooltip>
        <template #default="{ row }">
          {{ orderContent(row) }}
        </template>
      </el-table-column>
      <el-table-column align="center" label="金额" width="110">
        <template #default="{ row }">￥{{ (Number(row.amount || 0) / 100).toFixed(2) }}</template>
      </el-table-column>
      <el-table-column align="center" label="支付渠道" width="100">
        <template #default="{ row }">
          {{ Number(row.channel) === 1 ? '微信' : '支付宝' }}
        </template>
      </el-table-column>
      <el-table-column align="center" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row)">{{ statusText(row) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column align="center" label="支付时间" prop="paid_at" width="180">
        <template #default="{ row }">{{ row.paid_at || '—' }}</template>
      </el-table-column>
      <el-table-column align="center" label="创建时间" prop="created_at" width="180" />
      <el-table-column align="center" fixed="right" label="操作" width="160">
        <template #default="{ row }">
          <el-button v-if="isWaiting(row)" link :loading="payingNo === String(row.order_no)" type="primary" @click="continuePay(row)">
            继续支付
          </el-button>
          <el-button v-if="isWaiting(row)" link type="danger" @click="cancelOrder(row)">取消</el-button>
        </template>
      </el-table-column>
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

    <!-- 扫码支付 -->
    <pay-qr-dialog ref="payQrRef" @closed="fetchData" @paid="onPaid" />
  </div>
</template>

<script lang="ts" setup>
import { Operation, FolderDelete, FolderChecked } from '@element-plus/icons-vue'
import { ElMessageBox } from 'element-plus'
import PayQrDialog from '/@/views/centers/vabAutoComponents/PayQrDialog.vue'
import { creditsOrders } from '/@/api/credits'
import { payWechatQr, payAlipayQr, payCloseOrder } from '/@/api/pay'
import { getStorage } from '/@/utils/storage'
const $baseMessage = inject<any>('$baseMessage')

defineOptions({ name: 'CentersOrders' })

const queryForm = reactive({
  page: 1,
  pageSize: Number(getStorage('pageSize') || 10),
})
const listLoading = ref(false)
const rows = reactive({
  total: 0,
  list: [] as any[],
})
const layout = ref('prev, pager, next, total')

const TabsActiveName = ref('all')
const statusTabs = reactive([
  { label: '全部订单', value: 'all' },
  { label: '待支付', value: 'waiting' },
  { label: '已完成', value: 'paid' },
])

/* 状态判定：0 待支付 / 1 已支付 / 2 已关闭 / 3 已退款；待支付且已超时按过期展示 */
const orderStatus = (row: any): string => {
  const status = Number(row?.status ?? -1)
  if (status === 1) return 'paid'
  if (status === 2) return 'closed'
  if (status === 3) return 'refunded'
  if (status === 0) {
    const expire = row?.times_expire ? new Date(String(row.times_expire).replace(/-/g, '/')) : null
    if (expire && expire.getTime() < Date.now()) return 'expired'
    return 'waiting'
  }
  return 'closed'
}
const isWaiting = (row: any) => orderStatus(row) === 'waiting'
const statusText = (row: any) => {
  const map: Record<string, string> = { waiting: '待支付', paid: '已支付', closed: '已关闭', refunded: '已退款', expired: '已过期' }
  return map[orderStatus(row)] || '—'
}
const statusTagType = (row: any): 'primary' | 'success' | 'warning' | 'info' | 'danger' => {
  const map: Record<string, 'primary' | 'success' | 'warning' | 'info' | 'danger'> = {
    waiting: 'warning',
    paid: 'success',
    closed: 'info',
    refunded: 'danger',
    expired: 'info',
  }
  return map[orderStatus(row)] || 'info'
}
const sceneText = (scene: any) => (Number(scene) === 1 ? '订阅' : '积分充值')
const orderContent = (row: any) => {
  const ext = (row?.extension ?? {}) as Record<string, unknown>
  if (Number(row?.scene) === 1) return String(ext.plan_name || '全年订阅')
  const credits = Number(ext.credits ?? 0)
  return credits > 0 ? `${credits} 积分` : '—'
}

const fetchData = async () => {
  listLoading.value = true
  try {
    const { data }: any = await creditsOrders({ page: queryForm.page, pageSize: queryForm.pageSize })
    const list = Array.isArray(data?.list) ? data.list : []
    rows.list = TabsActiveName.value === 'all' ? list : list.filter((row: any) => orderStatus(row) === TabsActiveName.value)
    rows.total = Number(data?.total ?? 0)
  } catch (error) {
    console.error('[centersOrders] 订单列表加载失败:', error)
    rows.list = []
    rows.total = 0
  } finally {
    listLoading.value = false
  }
}

const changeTab = () => {
  queryForm.page = 1
  fetchData()
}

const handleCurrentChange = (value: number) => {
  queryForm.page = value
  fetchData()
}

/* ─── 继续支付 / 取消 ─── */
const payQrRef = ref<InstanceType<typeof PayQrDialog> | null>(null)
const payingNo = ref('')

const continuePay = async (row: any) => {
  const orderNo = String(row.order_no || '')
  if (!orderNo) return
  payingNo.value = orderNo
  try {
    // 二维码内容优先复用下单时的缓存，缺失/过期时按渠道重取
    const cached = String((row.extension ?? ({} as Record<string, unknown>))?.qr_content || '')
    let qrContent = cached
    if (!qrContent) {
      const fetcher = Number(row.channel) === 1 ? payWechatQr : payAlipayQr
      const { data }: any = await fetcher({ order_no: orderNo })
      qrContent = String(data?.qr_content || '')
    }
    if (!qrContent) {
      $baseMessage?.('获取支付二维码失败，请稍后重试', 'warning', 'hey')
      return
    }
    payQrRef.value?.open({
      orderNo,
      amount: Number(row.amount || 0),
      qrContent,
      channel: Number(row.channel || 1),
    })
  } catch (error) {
    console.error('[centersOrders] 继续支付失败:', error)
  } finally {
    payingNo.value = ''
  }
}

const cancelOrder = async (row: any) => {
  const orderNo = String(row.order_no || '')
  if (!orderNo) return
  try {
    await ElMessageBox.confirm('确定取消该待支付订单？取消后无法恢复，可重新下单。', '取消订单', {
      confirmButtonText: '确认取消',
      cancelButtonText: '再想想',
      type: 'warning',
    })
  } catch {
    return
  }
  try {
    await import('/@/api/pay').then(({ payCloseOrder }) => payCloseOrder({ order_no: orderNo }))
    $baseMessage?.('订单已取消', 'success', 'hey')
    await fetchData()
  } catch (error) {
    console.error('[centersOrders] 取消订单失败:', error)
  }
}

const onPaid = async () => {
  $baseMessage?.('支付成功', 'success', 'hey')
  await fetchData()
}

onBeforeMount(() => {
  fetchData()
})
</script>

<style lang="scss">
.orders-box {
  background: rgb(255, 255, 255);

  .orders-tabs {
    .tabs-label {
      display: flex;
      align-items: center;
      cursor: pointer;
      user-select: none;
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
  }

  .el-row {
    padding: 20px 0;
  }
}
</style>
