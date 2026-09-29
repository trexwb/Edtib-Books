<!--
 * @Author: trexwb
 * @Date: 2026-09-11
 * @LastEditTime: 2026-09-11
 * @FilePath: /books/web/src/views/centers/subscription/index.vue
 * @Description: 订阅中心：订阅状态 / 全年订阅套餐（全部产品标准 + 当年更新）/ 时长兑换码
 *   订阅权益：订阅有效期内可获取全部产品标准及当年标准更新；
 *   积分单独购买的标准保留购买时版本，不随更新同步（更新仅面向订阅用户）。
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved.
-->

<template>
  <div v-loading="statusLoading" class="list-container table-auto-height subscription-box">
    <!-- 订阅状态 -->
    <el-card class="status-card" shadow="never">
      <div class="status-row">
        <div class="status-main">
          <el-tag v-if="subscribed" effect="dark" size="large" type="success">订阅生效中</el-tag>
          <el-tag v-else-if="mySub.status === 'expired'" effect="dark" size="large" type="info">订阅已到期</el-tag>
          <el-tag v-else effect="dark" size="large" type="warning">未订阅</el-tag>
          <div class="status-info">
            <template v-if="mySub.end_at">
              <div class="status-date">有效期至 {{ mySub.end_at }}</div>
              <div class="status-days">
                剩余
                <span class="days-num">{{ mySub.days_left }}</span>
                天
              </div>
            </template>
            <div v-else class="status-date">订阅全年可获得全部产品标准及当年标准更新</div>
          </div>
        </div>
        <div class="status-actions">
          <el-button v-if="subscribed" :icon="RefreshRight" type="primary" @click="scrollToPlans">续订（时长顺延）</el-button>
          <el-button v-else :icon="Wallet" type="primary" @click="scrollToPlans">立即订阅</el-button>
          <el-button :icon="Ticket" @click="redeemVisible = true">兑换时长码</el-button>
        </div>
      </div>
      <div class="status-tip">
        订阅权益：全部产品标准 + 当年标准更新；材料标准 / 性能标准 / 表面标准免费开放，无需订阅或积分购买；
        积分单独购买的产品标准保留购买时版本，不随标准更新同步。
      </div>
    </el-card>

    <!-- 订阅套餐 -->
    <el-card id="plans-card" class="plans-card" shadow="never">
      <template #header>
        <div class="plans-header">
          <span>全年订阅套餐</span>
          <span class="plans-sub">支付完成后订阅时长在现有到期时间上顺延</span>
        </div>
      </template>
      <div v-loading="plansLoading" class="plan-list">
        <div
          v-for="plan in plans"
          :key="plan.id"
          class="plan-item"
          :class="{ active: Number(selectedPlanId) === Number(plan.id) }"
          @click="selectedPlanId = Number(plan.id)"
        >
          <div class="plan-name">{{ plan.name }}</div>
          <div class="plan-price">
            ￥{{ (Number(plan.price || 0) / 100).toFixed(2) }}
            <span v-if="plan.original_price" class="plan-original">￥{{ (Number(plan.original_price) / 100).toFixed(2) }}</span>
          </div>
          <div class="plan-duration">{{ plan.duration_days }} 天</div>
          <div v-if="plan.gift_credits > 0" class="plan-gift">赠送 {{ plan.gift_credits }} 积分</div>
          <div v-if="plan.description" class="plan-desc">{{ plan.description }}</div>
        </div>
        <el-empty v-if="!plansLoading && plans.length === 0" class="plan-empty" description="暂无可订阅套餐" />
      </div>
      <div class="channel-row">
        <span class="channel-label">支付方式</span>
        <el-radio-group v-model="channel">
          <el-radio :label="1">微信支付</el-radio>
          <el-radio :label="2">支付宝</el-radio>
        </el-radio-group>
        <el-button class="submit-btn" :disabled="!selectedPlanId" :loading="subscribeLoading" type="primary" @click="submitSubscribe">
          确认订阅
        </el-button>
      </div>
    </el-card>

    <!-- 兑换码 -->
    <el-dialog v-model="redeemVisible" :close-on-click-modal="false" title="兑换订阅时长码" width="420px">
      <el-alert
        class="redeem-tip"
        :closable="false"
        show-icon
        title="订阅时长码兑换后直接在现有到期时间上顺延；积分兑换码请前往「积分中心」兑换。"
        type="info"
      />
      <el-input v-model="redeemCode" clearable placeholder="请输入订阅时长兑换码" @keyup.enter="submitRedeem" />
      <template #footer>
        <el-button @click="redeemVisible = false">取消</el-button>
        <el-button :loading="redeemLoading" type="primary" @click="submitRedeem">确认兑换</el-button>
      </template>
    </el-dialog>

    <!-- 扫码支付 -->
    <pay-qr-dialog ref="payQrRef" @paid="onPaid" />
  </div>
</template>

<script lang="ts" setup>
import { RefreshRight, Wallet, Ticket } from '@element-plus/icons-vue'
import PayQrDialog from '/@/views/centers/vabAutoComponents/PayQrDialog.vue'
import { subscriptionPlans, subscriptionSubscribe, mySubscription, subscriptionRedeem } from '/@/api/subscription'
const $baseMessage = inject<any>('$baseMessage')

defineOptions({ name: 'CentersSubscription' })

/** 多语言 JSON 文本取值（names/description 为 { zh_cn: '...' } 结构） */
const jsonText = (value: any): string => {
  if (!value) return ''
  if (typeof value === 'string') return value
  return String(value.zh_cn || value.zh || Object.values(value)[0] || '')
}

/* ─── 订阅状态 ─── */
const statusLoading = ref(false)
const mySub = reactive<any>({
  active: false,
  plan: null,
  start_at: null,
  end_at: null,
  days_left: 0,
  status: 'none',
})
const subscribed = computed(() => !!mySub.active)

const fetchMySubscription = async () => {
  statusLoading.value = true
  try {
    const { data }: any = await mySubscription()
    Object.assign(mySub, data ?? {})
  } catch (error) {
    console.error('[centersSubscription] 订阅状态加载失败:', error)
  } finally {
    statusLoading.value = false
  }
}

/* ─── 套餐与下单 ─── */
const plansLoading = ref(false)
const plans = ref<any[]>([])
const selectedPlanId = ref<number>(0)
const channel = ref<number>(1)
const subscribeLoading = ref(false)
const payQrRef = ref<InstanceType<typeof PayQrDialog> | null>(null)

const fetchPlans = async () => {
  plansLoading.value = true
  try {
    const { data }: any = await subscriptionPlans()
    const list = (Array.isArray(data?.list) ? data.list : []).map((item: any) => ({
      id: Number(item?.id ?? 0),
      name: jsonText(item?.names) || `订阅 ${item?.duration_days ?? ''} 天`,
      price: Number(item?.price ?? 0),
      original_price: Number(item?.original_price ?? 0) || 0,
      duration_days: Number(item?.duration_days ?? 0),
      gift_credits: Number(item?.gift_credits ?? 0),
      description: jsonText(item?.description),
    }))
    plans.value = list
    if (list.length) selectedPlanId.value = Number(list[0]?.id ?? 0)
  } catch (error) {
    console.error('[centersSubscription] 订阅套餐加载失败:', error)
    $baseMessage?.('订阅套餐加载失败，请稍后重试', 'warning', 'hey')
  } finally {
    plansLoading.value = false
  }
}

const scrollToPlans = () => {
  document.getElementById('plans-card')?.scrollIntoView({ behavior: 'smooth' })
}

const submitSubscribe = async () => {
  if (!selectedPlanId.value) {
    $baseMessage?.('请先选择订阅套餐', 'warning', 'hey')
    return
  }
  subscribeLoading.value = true
  try {
    const { data }: any = await subscriptionSubscribe({ plan_id: selectedPlanId.value, channel: channel.value })
    if (data?.order_no && data?.qr_content) {
      payQrRef.value?.open({
        orderNo: String(data.order_no),
        amount: Number(data.amount || 0),
        qrContent: String(data.qr_content || ''),
        channel: channel.value,
      })
      $baseMessage?.('订阅订单已创建，请扫码完成支付', 'success', 'hey')
    } else {
      $baseMessage?.('订单创建成功，但未获取到支付二维码，请到「订单管理」继续支付', 'warning', 'hey')
    }
  } catch (error) {
    console.error('[centersSubscription] 订阅下单失败:', error)
  } finally {
    subscribeLoading.value = false
  }
}

const onPaid = async () => {
  $baseMessage?.('支付成功，订阅已生效', 'success', 'hey')
  await fetchMySubscription()
}

/* ─── 时长兑换码 ─── */
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
    const { data }: any = await subscriptionRedeem({ code })
    $baseMessage?.(`兑换成功，订阅顺延 ${data?.value ?? 0} 天`, 'success', 'hey')
    redeemCode.value = ''
    redeemVisible.value = false
    await fetchMySubscription()
  } catch (error) {
    console.error('[centersSubscription] 兑换码兑换失败:', error)
  } finally {
    redeemLoading.value = false
  }
}

onBeforeMount(() => {
  fetchMySubscription()
  fetchPlans()
})
</script>

<style lang="scss">
.subscription-box {
  background: rgb(255, 255, 255);

  .status-card {
    margin-bottom: 12px;

    .status-row {
      display: flex;
      flex-wrap: wrap;
      gap: 24px;
      align-items: center;
    }

    .status-main {
      display: flex;
      gap: 16px;
      align-items: center;
    }

    .status-date {
      font-size: 14px;
      color: #606266;
    }

    .status-days {
      margin-top: 4px;
      font-size: 13px;
      color: #909399;

      .days-num {
        font-size: 20px;
        font-weight: bold;
        color: #052965;
      }
    }

    .status-actions {
      margin-left: auto;
    }

    .status-tip {
      margin-top: 10px;
      font-size: 12px;
      line-height: 18px;
      color: #909399;
    }
  }

  .plans-card {
    .plans-header {
      display: flex;
      gap: 12px;
      align-items: baseline;
      font-weight: bold;
      color: #052965;

      .plans-sub {
        font-size: 12px;
        font-weight: normal;
        color: #909399;
      }
    }

    .plan-list {
      display: flex;
      flex-wrap: wrap;
      gap: 16px;
      min-height: 90px;
    }

    .plan-item {
      width: 200px;
      padding: 16px;
      text-align: center;
      cursor: pointer;
      border: 1px solid #dcdfe6;
      border-radius: 8px;

      &.active {
        background: #eaf0f0;
        border-color: #052965;
      }

      .plan-name {
        font-size: 16px;
        font-weight: bold;
        color: #052965;
      }

      .plan-price {
        margin-top: 8px;
        font-size: 22px;
        font-weight: bold;
        color: #e6a23c;

        .plan-original {
          margin-left: 6px;
          font-size: 13px;
          font-weight: normal;
          color: #c0c4cc;
          text-decoration: line-through;
        }
      }

      .plan-duration {
        margin-top: 6px;
        font-size: 13px;
        color: #606266;
      }

      .plan-gift {
        margin-top: 4px;
        font-size: 12px;
        color: #67c23a;
      }

      .plan-desc {
        margin-top: 6px;
        font-size: 12px;
        line-height: 18px;
        color: #909399;
      }
    }

    .plan-empty {
      width: 100%;
    }

    .channel-row {
      display: flex;
      align-items: center;
      margin-top: 16px;

      .channel-label {
        margin-right: 16px;
        color: #606266;
      }

      .submit-btn {
        margin-left: auto;
      }
    }
  }

  .redeem-tip {
    margin-bottom: 12px;
  }
}
</style>
