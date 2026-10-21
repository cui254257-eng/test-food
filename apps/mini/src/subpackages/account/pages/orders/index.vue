<script setup lang="ts">
import { computed, ref } from 'vue'
import { onPullDownRefresh, onShow } from '@dcloudio/uni-app'
import { getMyOrders, type OrderSummary } from '@/api/orders'

type OrderFilter = 'all' | 'pending_payment' | 'completed' | 'cancelled'

const orders = ref<OrderSummary[]>([])
const activeFilter = ref<OrderFilter>('all')
const loading = ref(false)
const loadError = ref('')
const needsLogin = ref(false)

const visibleOrders = computed(() => activeFilter.value === 'all'
  ? orders.value
  : orders.value.filter((order) => order.status === activeFilter.value))

function money(cents: number) {
  return `¥${(cents / 100).toFixed(2)}`
}

function statusLabel(status: string) {
  switch (status) {
    case 'pending_payment': return '待付款'
    case 'cancelled': return '已取消'
    case 'completed': return '已完成'
    case 'preparing': return '制作中'
    case 'ready': return '待取餐'
    case 'paid': return '已支付'
    default: return status || '处理中'
  }
}

function modeLabel(mode: string) {
  switch (mode) {
    case 'dine_in': return '堂食'
    case 'delivery': return '外卖配送'
    case 'takeaway': return '到店自取'
    default: return '用餐方式'
  }
}

function dateLabel(value: string) {
  return value.replace(/-/g, '.').slice(0, 16)
}

async function loadOrders() {
  if (!uni.getStorageSync('customer-token')) {
    needsLogin.value = true
    orders.value = []
    return
  }

  loading.value = true
  loadError.value = ''
  needsLogin.value = false
  try {
    orders.value = await getMyOrders()
  } catch (error) {
    if (!uni.getStorageSync('customer-token')) needsLogin.value = true
    loadError.value = error instanceof Error ? error.message : '订单加载失败，请稍后重试'
  } finally {
    loading.value = false
  }
}

function openLogin() {
  uni.navigateTo({ url: '/subpackages/account/pages/login/index' })
}

function goOrderMenu() {
  uni.switchTab({ url: '/pages/index/index' })
}

onShow(() => { void loadOrders() })
onPullDownRefresh(() => {
  void loadOrders().finally(() => uni.stopPullDownRefresh())
})
</script>

<template>
  <view class="orders-page">
    <view class="orders-heading">
      <text class="heading-kicker">HAOWEI · ORDER HISTORY</text>
      <text class="heading-title">我的订单</text>
      <text class="heading-caption">每一份认真制作，都值得好好回味</text>
    </view>

    <view v-if="loading && orders.length === 0" class="state-card">
      <view class="state-mark loading-mark">⋯</view>
      <text class="state-title">正在查询订单</text>
      <text class="state-caption">请稍等片刻</text>
    </view>

    <view v-else-if="needsLogin" class="state-card">
      <view class="state-mark">♙</view>
      <text class="state-title">登录后查看专属订单</text>
      <text class="state-caption">订单记录仅对当前微信账号可见</text>
      <view class="primary-button" @tap="openLogin">微信登录</view>
    </view>

    <view v-else-if="loadError && orders.length === 0" class="state-card">
      <view class="state-mark">!</view>
      <text class="state-title">暂时无法加载</text>
      <text class="state-caption">{{ loadError }}</text>
      <view class="primary-button" @tap="loadOrders">重新加载</view>
    </view>

    <template v-else>
      <scroll-view class="filter-scroll" scroll-x show-scrollbar="false">
        <view class="filter-list">
          <view
            v-for="filter in [
              { key: 'all', label: '全部订单' },
              { key: 'pending_payment', label: '待付款' },
              { key: 'completed', label: '已完成' },
              { key: 'cancelled', label: '已取消' },
            ]"
            :key="filter.key"
            class="filter-pill"
            :class="{ active: activeFilter === filter.key }"
            @tap="activeFilter = filter.key as OrderFilter"
          >
            {{ filter.label }}
          </view>
        </view>
      </scroll-view>

      <view v-if="loadError" class="inline-error" @tap="loadOrders">{{ loadError }} · 点击重试</view>

      <view v-if="visibleOrders.length" class="order-list">
        <view v-for="order in visibleOrders" :key="order.order_number" class="order-card">
          <view class="order-topline">
            <view class="order-mode"><view class="mode-dot" />{{ modeLabel(order.fulfillment_mode) }}</view>
            <text class="order-status" :class="`status-${order.status}`">{{ statusLabel(order.status) }}</text>
          </view>

          <view class="order-meta">
            <text>订单号 {{ order.order_number }}</text>
            <text>{{ dateLabel(order.created_at) }}</text>
          </view>

          <view class="order-items">
            <view v-for="(item, index) in order.items" :key="`${item.product_name}-${index}`" class="order-item">
              <view class="item-copy">
                <text class="item-name">{{ item.product_name }}</text>
                <text v-if="item.selected_options.length" class="item-options">
                  {{ item.selected_options.map((option) => option.option_name).join(' · ') }}
                </text>
              </view>
              <text class="item-quantity">× {{ item.quantity }}</text>
              <text class="item-price">{{ money(item.line_total_cents) }}</text>
            </view>
          </view>

          <view v-if="order.fulfillment_mode === 'dine_in' && order.table_number" class="delivery-note">
            桌号 {{ order.table_number }}
          </view>
          <view v-else-if="order.fulfillment_mode === 'delivery' && order.delivery_address" class="delivery-note">
            {{ order.delivery_recipient_name }} · {{ order.delivery_phone }}<br />{{ order.delivery_address }}
          </view>

          <view class="order-total">
            <text>{{ order.items.reduce((sum, item) => sum + item.quantity, 0) }} 件商品</text>
            <text>{{ order.payment_status === 'paid' ? '实付' : '合计' }} <strong>{{ money(order.total_cents) }}</strong></text>
          </view>
          <view v-if="order.discount_cents > 0" class="order-member-discount">会员折扣 -{{ money(order.discount_cents) }}</view>
          <view v-if="order.points_earned > 0" class="order-member-points">本单获得 {{ order.points_earned }} 积分</view>
        </view>
      </view>

      <view v-else class="state-card empty-card">
        <view class="state-mark">☕</view>
        <text class="state-title">{{ activeFilter === 'all' ? '还没有订单' : '暂无此状态订单' }}</text>
        <text class="state-caption">点一份喜欢的餐食，让今天更有滋味</text>
        <view class="primary-button" @tap="goOrderMenu">去点餐</view>
      </view>
    </template>
  </view>
</template>

<style scoped>
.orders-page { min-height: 100vh; box-sizing: border-box; padding: 24px 16px calc(32px + env(safe-area-inset-bottom)); background: #f5f2ec; color: #34291f; }
.orders-heading { display: flex; flex-direction: column; gap: 7px; margin: 0 2px 20px; }
.heading-kicker { color: #ad8b71; font-size: 9px; font-weight: 700; letter-spacing: 2px; }
.heading-title { color: #392d23; font-family: serif; font-size: 25px; font-weight: 750; letter-spacing: 1px; }
.heading-caption { color: #a59a8d; font-size: 10px; }
.filter-scroll { width: calc(100% + 32px); margin: 0 -16px 14px; white-space: nowrap; }
.filter-list { display: inline-flex; gap: 8px; padding: 0 16px; }
.filter-pill { padding: 8px 14px; border: 1px solid #e9e2d8; border-radius: 18px; background: #fffdfa; color: #86796b; font-size: 10px; }
.filter-pill.active { border-color: #b96d4b; background: #b96d4b; color: #fff; box-shadow: 0 5px 13px rgb(185 109 75 / 18%); }
.order-list { display: flex; flex-direction: column; gap: 13px; }
.order-card, .state-card { border: 1px solid #eee8df; border-radius: 17px; background: #fffdfa; box-shadow: 0 7px 22px rgb(54 41 29 / 4%); }
.order-card { padding: 14px 15px 12px; }
.order-topline, .order-meta, .order-total { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.order-topline { padding-bottom: 11px; border-bottom: 1px solid #f0ebe4; }
.order-mode { display: inline-flex; align-items: center; gap: 7px; color: #5e5348; font-size: 11px; font-weight: 650; }
.mode-dot { width: 7px; height: 7px; border-radius: 50%; background: #8d9a73; }
.order-status { color: #a66343; font-size: 10px; font-weight: 700; }
.status-cancelled { color: #9a948d; }
.status-completed { color: #638161; }
.order-meta { padding: 10px 0 7px; color: #aaa094; font-size: 9px; }
.order-items { display: flex; flex-direction: column; gap: 9px; padding: 4px 0 10px; }
.order-item { display: flex; align-items: flex-start; gap: 9px; }
.item-copy { display: flex; min-width: 0; flex: 1; flex-direction: column; gap: 4px; }
.item-name { overflow: hidden; color: #44372c; font-size: 11px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
.item-options { color: #aaa094; font-size: 9px; }
.item-quantity { color: #a49a8f; font-size: 10px; }
.item-price { min-width: 52px; color: #77695d; font-size: 10px; text-align: right; }
.delivery-note { margin: 1px 0 9px; padding: 8px 10px; border-radius: 9px; background: #f8f5ef; color: #8f8376; font-size: 9px; line-height: 1.7; }
.order-total { padding-top: 10px; border-top: 1px solid #f0ebe4; color: #a49a8f; font-size: 9px; }
.order-total strong { margin-left: 4px; color: #a65e40; font-size: 15px; }
.order-member-discount, .order-member-points { margin-top: 6px; color: #a35e43; font-size: 9px; text-align: right; }
.order-member-points { color: #8b7c64; }
.state-card { display: flex; min-height: 230px; align-items: center; flex-direction: column; justify-content: center; padding: 24px; text-align: center; }
.empty-card { margin-top: 9px; }
.state-mark { display: grid; width: 50px; height: 50px; place-items: center; margin-bottom: 12px; border-radius: 50%; background: #f8eee4; color: #b66d4c; font-family: serif; font-size: 22px; }
.loading-mark { color: #8a9a77; font-size: 30px; }
.state-title { color: #514237; font-size: 13px; font-weight: 700; }
.state-caption { margin-top: 7px; color: #aaa094; font-size: 10px; line-height: 1.6; }
.primary-button { display: grid; min-width: 112px; height: 36px; place-items: center; margin-top: 17px; padding: 0 16px; border-radius: 19px; background: #b96d4b; box-shadow: 0 6px 15px rgb(185 109 75 / 17%); color: #fff; font-size: 10px; font-weight: 700; }
.inline-error { margin: 0 0 12px; padding: 9px 11px; border-radius: 9px; background: #fff3ee; color: #ae6248; font-size: 10px; }
</style>
