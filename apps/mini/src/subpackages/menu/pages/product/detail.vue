<script setup lang="ts">
import { onLoad } from '@dcloudio/uni-app'
import type { Product } from '@shop/contracts'
import { computed, ref } from 'vue'
import { addCartLine, getCartLines } from '@/api/cart'
import { getProduct } from '@/api/products'

const product = ref<Product | null>(null)
const selectedSpecifications = ref<Record<string, string>>({})
const error = ref('')
const selectedPrice = computed(() => {
  if (!product.value) return 0
  return product.value.price + (product.value.specifications ?? []).reduce((total, group) => {
    const selectedName = selectedSpecifications.value[group.name]
    const option = group.options.find((item) => item.name === selectedName)
    return total + (option?.price_delta ?? 0)
  }, 0)
})

onLoad(async (query) => {
  const id = Number(query?.id)
  if (!Number.isInteger(id) || id < 1) {
    error.value = '菜品不存在或链接已失效'
    return
  }
  try {
    product.value = await getProduct(id)
    for (const group of product.value.specifications ?? []) {
      if (group.required && group.options.length) {
        selectedSpecifications.value[group.name] = group.options[0].name
      }
    }
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : '菜品加载失败'
  }
})

function addToCart(showToast = true) {
  if (!product.value) return false
  if (product.value.status !== 'active' || product.value.stock <= 0) {
    uni.showToast({ title: '该菜品暂不可购买', icon: 'none' })
    return false
  }
  const specifications = product.value.specifications ?? []
  const missingRequired = specifications.find((group) => group.required && !selectedSpecifications.value[group.name])
  if (missingRequired) {
    uni.showToast({ title: `请选择${missingRequired.name}`, icon: 'none' })
    return false
  }
  const selections = specifications
    .flatMap((group) => selectedSpecifications.value[group.name]
      ? [{ group_name: group.name, option_name: selectedSpecifications.value[group.name] }]
      : [])
  const key = `${product.value.id}:${selections.map((item) => `${item.group_name}：${item.option_name}`).join('|')}`
  const currentQuantity = getCartLines().find((line) => line.key === key)?.quantity ?? 0
  if (currentQuantity + 1 > product.value.stock) {
    uni.showToast({ title: `最多可点 ${product.value.stock} 份`, icon: 'none' })
    return false
  }
  const added = addCartLine({
    key,
    product_id: product.value.id,
    name: product.value.name,
    image_url: product.value.image_url,
    unit_price: selectedPrice.value,
    quantity: 1,
    stock: product.value.stock,
    selections,
  })
  if (!added) {
    uni.showToast({ title: `最多可点 ${product.value.stock} 份`, icon: 'none' })
    return false
  }
  if (showToast) {
    uni.showToast({ title: '已加入购物篮', icon: 'success' })
    setTimeout(() => uni.navigateBack(), 250)
  }
  return true
}

function checkoutNow() {
  if (!addToCart(false)) return
  uni.navigateTo({ url: '/pages/checkout/index' })
}

function selectSpecification(groupName: string, optionName: string) {
  selectedSpecifications.value = { ...selectedSpecifications.value, [groupName]: optionName }
}

function goBack() {
  uni.navigateBack()
}
</script>

<template>
  <view v-if="product" class="min-h-screen pb-24">
    <view class="product-hero flex h-64 items-center justify-center bg-linear-to-br from-orange-50 to-amber-100">
      <image v-if="product.image_url" class="product-detail-image" :src="product.image_url" mode="aspectFill" />
      <view v-else class="product-placeholder">{{ product.name.slice(0, 1) }}</view>
    </view>
    <view class="product-content -mt-4 rounded-t-3xl bg-white px-5 pb-8 pt-5">
      <text class="block text-xl font-bold text-stone-900">{{ product.name }}</text>
      <text class="detail-category">{{ product.category }}</text>
      <text class="mt-2 block text-sm leading-6 text-stone-500">{{ product.description || '暂无描述' }}</text>
      <view class="price-row"><text class="price-label">单价</text><text class="product-price">¥{{ selectedPrice.toFixed(2) }}</text></view>
      <view v-for="group in product.specifications ?? []" :key="group.name" class="specification-block">
        <view class="specification-title">{{ group.name }}<text v-if="group.required" class="required-label">必选</text></view>
        <view class="specification-options">
          <view
            v-for="option in group.options"
            :key="option.name"
            class="specification-option"
            :class="{ selected: selectedSpecifications[group.name] === option.name }"
            @tap="selectSpecification(group.name, option.name)"
          >
            {{ option.name }}<text v-if="option.price_delta > 0"> +¥{{ option.price_delta.toFixed(2) }}</text>
          </view>
        </view>
      </view>
      <view class="stock-row">
        <text>今日可售</text><text class="stock-value">{{ product.stock }} 份</text>
      </view>
    </view>
    <view class="detail-actions">
      <view class="add-button" :class="{ disabled: product.stock <= 0 || product.status !== 'active' }" @tap="addToCart">加入购物篮</view>
      <view class="checkout-button" :class="{ disabled: product.stock <= 0 || product.status !== 'active' }" @tap="checkoutNow">立即支付</view>
      <view class="back-menu" @tap="goBack">还要加菜</view>
    </view>
  </view>
  <view v-else class="flex min-h-screen flex-col items-center justify-center px-6 text-center">
    <text class="text-sm text-slate-500">{{ error || '正在加载菜品…' }}</text>
    <wd-button v-if="error" class="mt-4" size="small" plain @click="goBack">返回菜单</wd-button>
  </view>
</template>

<style scoped>
.product-hero { height: 270px; overflow: hidden; background: linear-gradient(145deg, #fbefdf, #f2dfc7); }
.product-detail-image { width: 100%; height: 100%; }
.product-placeholder { display: grid; width: 120px; height: 120px; place-items: center; border-radius: 38px; background: rgb(255 255 255 / 55%); color: #ac7957; font-size: 54px; box-shadow: 0 15px 40px rgb(94 61 31 / 8%); }
.product-content { position: relative; min-height: 48vh; padding-bottom: 110px; border-radius: 24px 24px 0 0; }
.detail-category { display: inline-block; margin-top: 8px; padding: 4px 8px; border-radius: 10px; background: #f6eee5; color: #a56648; font-size: 10px; }
.price-row { display: flex; align-items: baseline; gap: 8px; margin-top: 18px; }
.price-label { color: #a49d93; font-size: 10px; }
.product-price { color: #b9573b; font-size: 24px; font-weight: 750; }
.specification-block { margin-top: 22px; padding-top: 16px; border-top: 1px solid #f2eee8; }
.specification-title { margin-bottom: 10px; color: #423e37; font-size: 12px; font-weight: 700; }
.required-label { margin-left: 8px; padding: 2px 5px; border-radius: 6px; background: #fff1e9; color: #bd6446; font-size: 8px; font-weight: 500; }
.specification-options { display: flex; flex-wrap: wrap; gap: 8px; }
.specification-option { padding: 9px 13px; border: 1px solid #eee9e1; border-radius: 18px; background: #faf9f6; color: #77736b; font-size: 10px; }
.specification-option.selected { border-color: #c77855; background: #fbf0e8; color: #a35337; box-shadow: inset 0 0 0 1px rgb(199 120 85 / 10%); }
.stock-row { display: flex; align-items: center; justify-content: space-between; margin-top: 22px; padding-top: 15px; border-top: 1px solid #f2eee8; color: #9a948a; font-size: 10px; }
.stock-value { color: #5a7657; font-weight: 600; }
.detail-actions { position: fixed; z-index: 10; right: 0; bottom: 0; left: 0; display: flex; align-items: center; gap: 8px; padding: 10px 14px calc(10px + env(safe-area-inset-bottom)); border-top: 1px solid #f0ece5; background: rgb(255 255 255 / 97%); box-shadow: 0 -6px 24px rgb(49 39 22 / 5%); }
.detail-actions > view { display: grid; min-height: 42px; place-items: center; border-radius: 22px; font-size: 10px; font-weight: 700; }
.add-button { flex: 1.1; border: 1px solid #c77957; background: #fff; color: #a85e3f; }
.checkout-button { flex: 1; background: #bd704f; color: #fff; }
.back-menu { flex: 0 0 auto; padding: 0 7px; color: #898278; font-weight: 500 !important; }
.detail-actions .disabled { opacity: .45; }
</style>
