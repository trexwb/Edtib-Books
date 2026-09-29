<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/centers/vabAutoComponents/PayQrDialog.vue
 * @Description: 扫码支付弹窗（积分充值 / 订阅共用）
 *   open(orderNo, amount, qrContent, channel) 后 2s 轮询 /front/pay/queryOrder；
 *   支付成功 emit('paid')，取消订单 emit('closed')；二维码内容优先使用下单返回值，
 *   继续支付场景由调用方重取后传入。
-->

<template>
  <el-dialog v-model="visible" :close-on-click-modal="false" :title="dialogTitle" width="420px" @close="stopPolling">
    <div v-loading="loading" class="pay-qr-box">
      <div class="pay-amount">￥{{ amountText }}</div>
      <div class="pay-order-no">订单号 {{ orderNo || '—' }}</div>
      <vab-qr-code v-if="qrContent" class="pay-qr" :size="220" :text="qrContent" />
      <div class="pay-tip">{{ payTip }}</div>
      <div class="pay-status">
        <el-tag v-if="payStatus === 'waiting'" effect="dark" type="warning">待支付</el-tag>
        <el-tag v-else-if="payStatus === 'paid'" effect="dark" type="success">已支付</el-tag>
        <el-tag v-else-if="payStatus === 'closed'" effect="dark" type="info">已关闭</el-tag>
        <el-tag v-else effect="dark" type="danger">已过期</el-tag>
      </div>
    </div>
    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
      <el-button v-if="payStatus === 'waiting'" :loading="closing" type="danger" @click="closeOrder">取消订单</el-button>
    </template>
  </el-dialog>
</template>

<script lang="ts" setup>
import VabQrCode from '/@/plugins/VabQrCode'
import { payQueryOrder, payCloseOrder } from '/@/api/pay'

defineOptions({ name: 'PayQrDialog' })

const emit = defineEmits<{
  (e: 'paid', orderNo: string): void
  (e: 'closed', orderNo: string): void
}>()

const visible = ref(false)
const loading = ref(false)
const closing = ref(false)
const orderNo = ref('')
const amount = ref(0)
const qrContent = ref('')
const channel = ref(1)
const payStatus = ref<'waiting' | 'paid' | 'closed' | 'expired'>('waiting')
let pollTimer: ReturnType<typeof setInterval> | null = null

const amountText = computed(() => (Number(amount.value || 0) / 100).toFixed(2))
const channelText = computed(() => (Number(channel.value) === 1 ? '微信' : '支付宝'))
const dialogTitle = computed(() => `扫码支付（${channelText.value}）`)
const payTip = computed(() => {
  if (payStatus.value === 'paid') return '支付成功，积分/订阅权益已到账'
  if (payStatus.value === 'waiting') return `请使用${channelText.value}扫描二维码完成支付`
  if (payStatus.value === 'expired') return '订单已超时过期，请重新下单'
  return '订单已关闭'
})

/** 打开弹窗并开始轮询（amount 单位分） */
const open = (data: { orderNo: string; amount: number; qrContent: string; channel?: number }) => {
  orderNo.value = String(data.orderNo || '')
  amount.value = Number(data.amount || 0)
  qrContent.value = String(data.qrContent || '')
  channel.value = Number(data.channel || 1)
  payStatus.value = 'waiting'
  visible.value = true
  startPolling()
}

const stopPolling = () => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

const startPolling = () => {
  stopPolling()
  pollTimer = setInterval(async () => {
    if (!orderNo.value) return
    try {
      const { data }: any = await payQueryOrder({ order_no: orderNo.value })
      const status = String(data?.status || 'waiting')
      if (status !== 'waiting') {
        payStatus.value = status as typeof payStatus.value
        stopPolling()
        if (status === 'paid') emit('paid', orderNo.value)
      }
    } catch (error) {
      console.error('[payQrDialog] 支付状态查询失败:', error)
    }
  }, 2000)
}

const closeOrder = async () => {
  if (!orderNo.value) return
  closing.value = true
  try {
    const { data }: any = await payCloseOrder({ order_no: orderNo.value })
    payStatus.value = String(data?.status || 'closed') as typeof payStatus.value
    stopPolling()
    emit('closed', orderNo.value)
  } catch (error) {
    console.error('[payQrDialog] 关闭订单失败:', error)
  } finally {
    closing.value = false
  }
}

const handleClose = () => {
  stopPolling()
  visible.value = false
}

defineExpose({ open })

onBeforeUnmount(() => {
  stopPolling()
})
</script>

<style lang="scss" scoped>
.pay-qr-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0 4px;

  .pay-amount {
    font-size: 28px;
    font-weight: bold;
    color: #052965;
  }

  .pay-order-no {
    margin: 6px 0 14px;
    font-size: 12px;
    color: #909399;
  }

  .pay-qr {
    padding: 8px;
    background: #fff;
    border: 1px solid #e4e7ed;
    border-radius: 6px;
  }

  .pay-tip {
    margin-top: 14px;
    font-size: 13px;
    color: #606266;
  }

  .pay-status {
    margin-top: 10px;
  }
}
</style>
