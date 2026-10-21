<script setup lang="ts">
import { onShow } from '@dcloudio/uni-app'
import { computed, ref } from 'vue'
import { getCartLines, setCartLineQuantity, type CartLine } from '@/api/cart'

const lines = ref<CartLine[]>([])
const itemCount = computed(() => lines.value.reduce((sum, line) => sum + line.quantity, 0))
const totalCents = computed(() => lines.value.reduce(
  (sum, line) => sum + Math.round(line.unit_price * 100) * line.quantity,
  0,
))

function refreshCart() {
  lines.value = getCartLines()
}

function changeQuantity(line: CartLine, delta: number) {
  lines.value = setCartLineQuantity(line.key, line.quantity + delta)
}

function removeLine(line: CartLine) {
  lines.value = setCartLineQuantity(line.key, 0)
}

function formatSelections(line: CartLine) {
  return line.selections.map((selection) => `${selection.group_name}：${selection.option_name}`).join(' · ')
}

function goCheckout() {
  uni.navigateTo({ url: '/pages/checkout/index' })
}

function goMenu() {
  const mode = String(uni.getStorageSync('restaurant-fulfillment-mode') || 'takeaway')
  uni.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${encodeURIComponent(mode)}` })
}

onShow(refreshCart)
</script>

<template>
  <view class="cart-page min-h-screen px-4 pb-8 pt-5">
    <view class="page-heading">
      <view>
        <text class="heading-kicker">YOUR SELECTION</text>
        <text class="page-title">购物车</text>
        <text class="page-subtitle">已选好的美味都在这里</text>
      </view>
      <view class="heading-icon">🛍</view>
    </view>

    <view v-if="lines.length" class="cart-list">
      <view class="list-caption"><text>商品清单</text><text>共 {{ itemCount }} 份</text></view>
      <view v-for="line in lines" :key="line.key" class="cart-item">
        <image v-if="line.image_url" class="item-image" :src="line.image_url" mode="aspectFill" />
        <view v-else class="item-placeholder">{{ line.name.slice(0, 1) }}</view>
        <view class="item-info">
          <text class="item-name">{{ line.name }}</text>
          <text v-if="line.selections.length" class="item-selections">{{ formatSelections(line) }}</text>
          <text class="item-price">¥{{ line.unit_price.toFixed(2) }}</text>
        </view>
        <view class="quantity-control">
          <text class="quantity-button" @tap="changeQuantity(line, -1)">−</text>
          <text class="quantity-value">{{ line.quantity }}</text>
          <text class="quantity-button plus" :class="{ disabled: line.quantity >= line.stock }" @tap="changeQuantity(line, 1)">＋</text>
        </view>
        <text class="remove-action" @tap="removeLine(line)">移除</text>
      </view>
    </view>

    <view v-else class="empty-card">
      <text class="empty-icon">🥣</text>
      <text class="empty-title">购物车还是空的</text>
      <text class="empty-copy">去今日菜单挑几道喜欢的菜吧</text>
      <view class="menu-button" @tap="goMenu">去今日菜单</view>
    </view>

    <view v-if="lines.length" class="cart-summary">
      <view class="summary-row"><text>商品数量</text><text>{{ itemCount }} 份</text></view>
      <view class="summary-row"><text>商品金额</text><text class="summary-price">¥{{ (totalCents / 100).toFixed(2) }}</text></view>
      <text class="summary-note">配送方式和配送费将在结账页确认</text>
      <view class="checkout-button" @tap="goCheckout">去结账 <text>¥{{ (totalCents / 100).toFixed(2) }}</text></view>
    </view>
  </view>
</template>

<style scoped>
.cart-page { box-sizing: border-box; min-height: 100vh; padding-bottom: calc(32px + env(safe-area-inset-bottom)); background: #f6f4f0; color: #302c26; }
.page-heading { display: flex; align-items: center; justify-content: space-between; margin: 2px 2px 22px; }
.page-heading > view:first-child { display: flex; flex-direction: column; }
.heading-kicker { color: #b47a5c; font-size: 8px; font-weight: 700; letter-spacing: 1.6px; }
.page-title { margin-top: 5px; color: #302c26; font-size: 23px; font-weight: 750; }
.page-subtitle { margin-top: 4px; color: #9b958b; font-size: 10px; }
.heading-icon { display: grid; width: 44px; height: 44px; place-items: center; border: 1px solid #eee7dc; border-radius: 15px; background: #fff; font-size: 21px; box-shadow: 0 5px 16px rgb(49 39 22 / 4%); }
.list-caption { display: flex; justify-content: space-between; margin: 0 3px 10px; color: #8e887e; font-size: 10px; }
.cart-item, .empty-card, .cart-summary { border: 1px solid rgb(231 225 215 / 70%); border-radius: 17px; background: #fff; box-shadow: 0 7px 24px rgb(49 39 22 / 4%); }
.cart-list { display: flex; flex-direction: column; gap: 9px; }
.cart-item { position: relative; display: flex; min-height: 82px; align-items: center; gap: 11px; padding: 13px; }
.item-image, .item-placeholder { width: 60px; height: 60px; flex: 0 0 60px; border-radius: 13px; }
.item-image { background: #f6f1e9; }
.item-placeholder { display: grid; place-items: center; background: linear-gradient(145deg, #f8ebdd, #f1dcc8); color: #aa704d; font-size: 22px; }
.item-info { display: flex; min-width: 0; flex: 1; flex-direction: column; gap: 5px; padding-bottom: 8px; }
.item-name { overflow: hidden; color: #38352f; font-size: 12px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
.item-selections { overflow: hidden; color: #9b958c; font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.item-price { color: #b9573b; font-size: 12px; font-weight: 700; }
.quantity-control { display: flex; align-items: center; gap: 7px; align-self: flex-end; margin-bottom: 2px; }
.quantity-button { display: grid; width: 25px; height: 25px; place-items: center; border-radius: 50%; background: #f3f0ea; color: #706b62; font-size: 15px; }
.quantity-button.plus { background: #bd7555; color: #fff; }
.quantity-button.disabled { opacity: .4; }
.quantity-value { min-width: 12px; color: #4c4841; font-size: 10px; text-align: center; }
.remove-action { position: absolute; top: 12px; right: 14px; color: #aaa399; font-size: 8px; }
.empty-card { display: flex; min-height: 250px; align-items: center; flex-direction: column; justify-content: center; padding: 30px 20px; text-align: center; }
.empty-icon { font-size: 40px; }
.empty-title { margin-top: 13px; color: #454139; font-size: 14px; font-weight: 700; }
.empty-copy { margin-top: 7px; color: #9b968e; font-size: 10px; }
.menu-button { margin-top: 17px; padding: 11px 25px; border-radius: 22px; background: #bb6d4c; color: #fff; font-size: 11px; font-weight: 700; }
.cart-summary { display: flex; flex-direction: column; gap: 13px; margin-top: 15px; padding: 16px; }
.summary-row { display: flex; align-items: center; justify-content: space-between; color: #8d877e; font-size: 10px; }
.summary-row text:last-child { color: #59544c; font-weight: 600; }
.summary-row .summary-price { color: #b9573b; font-size: 17px; font-weight: 750; }
.summary-note { color: #aaa399; font-size: 8px; }
.checkout-button { display: flex; min-height: 44px; align-items: center; justify-content: center; gap: 8px; border-radius: 23px; background: #b96d4b; color: #fff; font-size: 11px; font-weight: 700; }
.checkout-button text { font-size: 10px; font-weight: 500; }
</style>
