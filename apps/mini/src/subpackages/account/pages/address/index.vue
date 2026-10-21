<script setup lang="ts">
import { onLoad, onShow } from '@dcloudio/uni-app'
import { ref } from 'vue'
import { deleteAddress, getAddresses, type Address } from '@/api/addresses'

const addresses = ref<Address[]>([])
const loading = ref(false)
const errorMessage = ref('')
const selecting = ref(false)
const selectedAddressId = ref(0)

onLoad((query) => {
  selecting.value = query?.select === '1'
  selectedAddressId.value = Number(uni.getStorageSync('restaurant-selected-address-id')) || 0
})

function openEditor(id?: number) {
  uni.navigateTo({ url: id ? `/subpackages/account/pages/address/edit?id=${id}` : '/subpackages/account/pages/address/edit' })
}

function chooseAddress(address: Address) {
  if (!selecting.value) {
    openEditor(address.id)
    return
  }
  selectedAddressId.value = address.id
  uni.setStorageSync('restaurant-selected-address-id', address.id)
  uni.navigateBack()
}

function maskPhone(phone: string) {
  return phone.length === 11 ? `${phone.slice(0, 3)}****${phone.slice(7)}` : phone
}

async function loadAddresses() {
  loading.value = true
  errorMessage.value = ''
  try {
    addresses.value = await getAddresses()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '收货地址加载失败，请稍后重试'
  } finally {
    loading.value = false
  }
}

function removeAddress(address: Address) {
  uni.showModal({
    title: '删除收货地址',
    content: `确定删除 ${address.recipient_name} 的收货地址吗？`,
    confirmColor: '#b86a49',
    success: async (result) => {
      if (!result.confirm) return
      try {
        await deleteAddress(address.id)
        uni.showToast({ title: '已删除', icon: 'success' })
        await loadAddresses()
      } catch (error) {
        uni.showToast({
          title: error instanceof Error ? error.message : '删除失败，请稍后重试',
          icon: 'none',
        })
      }
    },
  })
}

onShow(() => {
  void loadAddresses()
})
</script>

<template>
  <view class="address-page min-h-screen px-4 pb-10 pt-4">
    <view v-if="selecting" class="selection-heading">选择外卖地址</view>
    <view class="privacy-note">
      <text class="privacy-icon">🔒</text>
      <text>地址仅用于配送，并与当前微信账号绑定</text>
    </view>

    <view v-if="loading && addresses.length === 0" class="state-card">正在加载收货地址…</view>
    <view v-else-if="errorMessage && addresses.length === 0" class="state-card">
      <text class="state-title">暂时无法加载</text>
      <text class="state-copy">{{ errorMessage }}</text>
      <view class="retry-button" @tap="loadAddresses">重新加载</view>
    </view>
    <view v-else-if="addresses.length === 0" class="empty-card">
      <view class="empty-mark">⌖</view>
      <text class="empty-title">还没有收货地址</text>
      <text class="empty-copy">添加外卖配送地址</text>
    </view>

    <view v-else class="address-list">
      <view v-for="address in addresses" :key="address.id" class="address-card">
        <view class="address-main" @tap="chooseAddress(address)">
          <view class="recipient-line">
            <text class="recipient-name">{{ address.recipient_name }}</text>
            <text class="recipient-phone">{{ maskPhone(address.phone) }}</text>
            <text v-if="address.is_default" class="default-tag">默认</text>
            <text v-if="selecting && selectedAddressId === address.id" class="selected-tag">已选</text>
          </view>
          <text class="address-detail">
            {{ address.province }}{{ address.city }}{{ address.district }}{{ address.detail }}
          </text>
        </view>
        <view class="address-actions">
          <text class="edit-action" @tap="openEditor(address.id)">编辑</text>
          <text class="action-separator">·</text>
          <text class="delete-action" @tap="removeAddress(address)">删除</text>
        </view>
      </view>
    </view>

    <view class="add-button" @tap="openEditor()">
      <text class="add-symbol">＋</text>
      <text>新增收货地址</text>
    </view>
    <text v-if="errorMessage && addresses.length > 0" class="inline-error">{{ errorMessage }}</text>
  </view>
</template>

<style scoped>
.address-page { box-sizing: border-box; background: #f4f2ed; color: #302c26; }
.selection-heading { margin: 2px 0 12px; color: #312d27; font-size: 18px; font-weight: 800; }
.privacy-note { display: flex; align-items: center; gap: 7px; padding: 11px 13px; border-radius: 12px; background: #f8f0e7; color: #8e6c54; font-size: 11px; }
.privacy-icon { font-size: 13px; }
.address-list { display: flex; flex-direction: column; gap: 11px; margin-top: 14px; }
.address-card { overflow: hidden; border-radius: 16px; background: #fff; box-shadow: 0 5px 18px rgb(49 39 22 / 4%); }
.address-main { padding: 17px 16px 13px; }
.recipient-line { display: flex; align-items: center; gap: 11px; }
.recipient-name { color: #312d27; font-size: 15px; font-weight: 700; }
.recipient-phone { color: #716d65; font-size: 12px; }
.default-tag { padding: 3px 7px; border-radius: 6px; background: #f9eee6; color: #b86a49; font-size: 10px; }
.selected-tag { margin-left: auto; padding: 3px 7px; border-radius: 6px; background: #eaf4e7; color: #56824e; font-size: 10px; }
.address-detail { display: block; margin-top: 10px; color: #77736b; font-size: 12px; line-height: 1.6; }
.address-actions { display: flex; align-items: center; justify-content: flex-end; gap: 9px; padding: 10px 16px 12px; border-top: 1px solid #f3f1ed; color: #89857d; font-size: 11px; }
.edit-action { color: #706b62; }
.delete-action { color: #b8765c; }
.action-separator { color: #d0ccc3; }
.add-button { display: flex; align-items: center; justify-content: center; gap: 7px; margin-top: 17px; padding: 13px; border-radius: 13px; background: #b86a49; color: #fff; font-size: 13px; font-weight: 600; box-shadow: 0 7px 16px rgb(184 106 73 / 18%); }
.add-symbol { font-size: 18px; line-height: 1; }
.empty-card, .state-card { display: flex; align-items: center; flex-direction: column; margin-top: 16px; padding: 35px 22px; border-radius: 16px; background: #fff; text-align: center; }
.empty-mark { display: grid; width: 52px; height: 52px; place-items: center; border-radius: 50%; background: #f8f1e9; color: #b86a49; font-size: 28px; }
.empty-title, .state-title { margin-top: 14px; color: #454139; font-size: 14px; font-weight: 700; }
.empty-copy, .state-copy { margin-top: 7px; color: #969188; font-size: 11px; line-height: 1.6; }
.state-card { color: #969188; font-size: 12px; line-height: 1.6; }
.retry-button { margin-top: 13px; padding: 8px 15px; border-radius: 18px; background: #f8f1e9; color: #a45f42; font-size: 11px; }
.inline-error { display: block; margin-top: 10px; color: #a45f42; font-size: 11px; text-align: center; }
</style>
