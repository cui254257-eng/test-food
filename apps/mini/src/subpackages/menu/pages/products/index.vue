<script setup lang="ts">
import { onLoad, onPullDownRefresh, onShow } from "@dcloudio/uni-app";
import type { Product } from "@shop/contracts";
import { computed, ref } from "vue";
import { getCartLines, setCartLineQuantity, type CartLine } from "@/api/cart";
import { getActiveProducts } from "@/api/products";

type SortMode = "newest" | "price";

const products = ref<Product[]>([]);
const keyword = ref("");
const sortMode = ref<SortMode>("newest");
const selectedCategory = ref("全部");
const loading = ref(false);
const error = ref("");
const cartLines = ref<CartLine[]>([]);
const cartExpanded = ref(false);

const categories = computed(() => [
  "全部",
  ...new Set(products.value.map((product) => product.category).filter(Boolean)),
]);

const sortedProducts = computed(() => {
  const list = products.value.filter(
    (product) =>
      selectedCategory.value === "全部" ||
      product.category === selectedCategory.value,
  );
  if (sortMode.value === "price")
    return [...list].sort((a, b) => a.price - b.price);
  return [...list].sort((a, b) => b.id - a.id);
});
const cartCount = computed(() => cartLines.value.reduce((sum, line) => sum + line.quantity, 0));
const cartTotalCents = computed(() => cartLines.value.reduce((sum, line) => sum + Math.round(line.unit_price * 100) * line.quantity, 0));

function money(cents: number) {
  return (cents / 100).toFixed(2);
}

function refreshCart() {
  cartLines.value = getCartLines();
  if (!cartLines.value.length) cartExpanded.value = false;
}

function updateLine(line: CartLine, delta: number) {
  cartLines.value = setCartLineQuantity(line.key, line.quantity + delta);
  if (!cartLines.value.length) cartExpanded.value = false;
}

function formatSelections(line: CartLine) {
  return line.selections.map((selection) => `${selection.group_name}：${selection.option_name}`).join(' · ');
}

function goCheckout() {
  uni.navigateTo({ url: '/pages/checkout/index' });
}

async function loadProducts() {
  loading.value = true;
  error.value = "";
  try {
    products.value = await getActiveProducts(keyword.value);
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : "菜单加载失败";
  } finally {
    loading.value = false;
    uni.stopPullDownRefresh();
  }
}

function openProduct(product: Product) {
  uni.navigateTo({ url: `/subpackages/menu/pages/product/detail?id=${product.id}` });
}

function goBack() {
  uni.navigateBack();
}

onLoad((query) => {
  keyword.value = String(query?.q ?? "");
  const mode = String(query?.mode ?? "");
  if (mode === "dine_in" || mode === "takeaway" || mode === "delivery") {
    uni.setStorageSync("restaurant-fulfillment-mode", mode);
  }
  const sort = String(query?.sort ?? "");
  if (sort === "price" || sort === "newest") sortMode.value = sort;
  void loadProducts();
});
onPullDownRefresh(() => {
  void loadProducts();
});
onShow(refreshCart);
</script>

<template>
  <view class="catalog-page min-h-screen px-4 pb-8 pt-4" :class="{ 'has-cart': cartCount > 0 }">
    <view class="catalog-search">
      <wd-icon name="search" size="17px" color="#8b8a84" />
      <input
        v-model="keyword"
        placeholder="搜索菜品、饮品"
        confirm-type="search"
        @confirm="loadProducts()"
      />
      <text
        v-if="keyword"
        class="catalog-clear"
        @click="
          keyword = '';
          loadProducts();
        "
        >清除</text
      >
    </view>

    <view class="sort-row">
      <scroll-view class="category-scroll" scroll-x :show-scrollbar="false">
        <view class="category-row">
          <text
            v-for="category in categories"
            :key="category"
            class="category-chip"
            :class="{ chosen: selectedCategory === category }"
            @tap="selectedCategory = category"
            >{{ category }}</text
          >
        </view>
      </scroll-view>
      <view class="sort-options">
        <text
          :class="{ selected: sortMode === 'newest' }"
          @click="sortMode = 'newest'"
          >推荐</text
        >
        <text
          :class="{ selected: sortMode === 'price' }"
          @click="sortMode = 'price'"
          >价格</text
        >
      </view>
    </view>

    <view v-if="loading && products.length === 0" class="catalog-empty"
      >正在加载菜单…</view
    >
    <view v-else-if="error" class="catalog-empty">
      <text>{{ error }}</text>
      <wd-button class="mt-3" size="small" plain @click="loadProducts"
        >重试</wd-button
      >
    </view>
    <view v-else-if="sortedProducts.length === 0" class="catalog-empty"
      >暂时没有找到相关菜品</view
    >
    <view v-else class="catalog-grid">
      <view
        v-for="product in sortedProducts"
        :key="product.id"
        class="catalog-card"
        @click="openProduct(product)"
      >
        <view class="catalog-art">
          <image
            v-if="product.image_url"
            :src="product.image_url"
            mode="aspectFill"
          />
          <text v-else>{{ product.name.slice(0, 1) }}</text>
        </view>
        <view class="catalog-info">
          <text class="catalog-category">{{ product.category }}</text>
          <text class="block truncate text-sm font-semibold text-stone-900">{{
            product.name
          }}</text>
          <text class="catalog-description mt-1 block truncate">{{
            product.description || "—"
          }}</text>
          <text class="catalog-price">¥{{ product.price.toFixed(2) }}</text>
          <text v-if="product.discount_eligible" class="catalog-member-tag">会员折扣</text>
        </view>
      </view>
    </view>

    <view v-if="cartExpanded" class="cart-scrim" @tap="cartExpanded = false" />
    <view v-if="cartExpanded" class="cart-sheet">
      <view class="sheet-heading">
        <view><text class="sheet-title">已选菜品</text><text class="sheet-caption">共 {{ cartCount }} 份</text></view>
        <text class="sheet-close" @tap="cartExpanded = false">收起 ˅</text>
      </view>
      <scroll-view class="cart-lines" scroll-y>
        <view v-for="line in cartLines" :key="line.key" class="cart-line">
          <view class="line-art">
            <image v-if="line.image_url" :src="line.image_url" mode="aspectFill" />
            <text v-else>{{ line.name.slice(0, 1) }}</text>
          </view>
          <view class="line-info">
            <text class="line-name">{{ line.name }}</text>
            <text v-if="line.selections.length" class="line-selections">{{ formatSelections(line) }}</text>
            <text class="line-price">¥{{ line.unit_price.toFixed(2) }}</text>
          </view>
          <view class="line-quantity">
            <text class="quantity-button" @tap="updateLine(line, -1)">−</text>
            <text class="quantity-value">{{ line.quantity }}</text>
            <text class="quantity-button plus" :class="{ disabled: line.quantity >= line.stock }" @tap="updateLine(line, 1)">＋</text>
          </view>
        </view>
      </scroll-view>
    </view>
    <view v-if="cartCount" class="cart-dock">
      <view class="cart-summary">
        <view class="cart-bag" @tap.stop="cartExpanded = !cartExpanded"><text>🛍</text><text class="cart-badge">{{ cartCount }}</text></view>
        <view class="cart-total-copy"><text class="cart-total-label">购物篮 · {{ cartCount }} 份</text><text class="cart-total-price">¥{{ money(cartTotalCents) }}</text></view>
      </view>
      <view class="checkout-action" @tap="goCheckout">立即支付</view>
    </view>
  </view>
</template>

<style scoped>
.catalog-page {
  background: #f4f2ed;
  color: #26241f;
}
.catalog-page.has-cart { padding-bottom: 105px; }
.catalog-heading {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 4px 0 18px;
}
.catalog-back {
  display: grid;
  width: 36px;
  height: 36px;
  flex: 0 0 36px;
  place-items: center;
  border-radius: 50%;
  background: #fff;
}
.catalog-count {
  display: block;
  margin-top: 3px;
  color: #9a958b;
  font-size: 10px;
}
.catalog-search {
  display: flex;
  height: 42px;
  align-items: center;
  gap: 9px;
  padding: 0 14px;
  border-radius: 23px;
  background: #fff;
}
.catalog-search input {
  min-width: 0;
  flex: 1;
  color: #514f48;
  font-size: 12px;
}
.catalog-clear {
  color: #ba6340;
  font-size: 11px;
}
.sort-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 15px 0 13px;
  color: #8d887f;
  font-size: 11px;
}
.category-scroll {
  min-width: 0;
  flex: 1;
  white-space: nowrap;
}
.category-row {
  display: inline-flex;
  gap: 7px;
}
.category-chip {
  padding: 7px 10px;
  border-radius: 14px;
  background: #fff;
  color: #8d887f;
}
.category-chip.chosen {
  background: #b86a49;
  color: #fff;
}
.sort-options {
  display: flex;
  flex: 0 0 auto;
  gap: 12px;
}
.sort-options text.selected {
  color: #b9573b;
  font-weight: 700;
}
.catalog-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}
.catalog-card {
  overflow: hidden;
  border-radius: 18px;
  background: #fff;
  box-shadow: 0 5px 18px rgb(49 39 22 / 4%);
}
.catalog-art {
  display: grid;
  height: 174rpx;
  place-items: center;
  overflow: hidden;
  background: #e9ebe1;
  color: #6f7867;
  font-size: 56rpx;
}
.catalog-art image {
  width: 100%;
  height: 100%;
}
.catalog-info {
  padding: 11px 12px 13px;
}
.catalog-category {
  display: inline-block;
  margin-bottom: 5px;
  padding: 3px 6px;
  border-radius: 8px;
  background: #f7f0e8;
  color: #a66a4c;
  font-size: 9px;
}
.catalog-description {
  color: #aaa69d;
  font-size: 10px;
}
.catalog-price {
  display: block;
  margin-top: 11px;
  color: #b9573b;
  font-size: 16px;
  font-weight: 700;
}
.catalog-member-tag { display: inline-block; margin-top: 5px; padding: 3px 6px; border-radius: 8px; background: #f7f0e5; color: #9a7849; font-size: 8px; }
.catalog-empty {
  display: flex;
  min-height: 140px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 18px;
  background: #fff;
  color: #969187;
  font-size: 12px;
}
.cart-scrim { position: fixed; z-index: 20; inset: 0; background: rgb(35 31 26 / 30%); }
.cart-sheet { position: fixed; z-index: 21; right: 12px; bottom: calc(72px + env(safe-area-inset-bottom)); left: 12px; max-height: 58vh; overflow: hidden; border: 1px solid #eee8df; border-radius: 20px; background: #fff; box-shadow: 0 14px 42px rgb(35 31 26 / 17%); }
.sheet-heading { display: flex; align-items: center; justify-content: space-between; padding: 16px 17px 12px; border-bottom: 1px solid #f2eee8; }
.sheet-heading > view { display: flex; align-items: baseline; gap: 8px; }
.sheet-title { color: #37332d; font-size: 13px; font-weight: 750; }
.sheet-caption, .sheet-close { color: #9c958a; font-size: 9px; }
.cart-lines { max-height: 45vh; padding: 0 16px; box-sizing: border-box; }
.cart-line { display: flex; align-items: center; gap: 10px; padding: 12px 0; border-bottom: 1px solid #f4f1ec; }
.cart-line:last-child { border-bottom: 0; }
.line-art { display: grid; width: 48px; height: 48px; flex: 0 0 48px; place-items: center; overflow: hidden; border-radius: 11px; background: #f5ebdf; color: #aa704d; font-size: 19px; }
.line-art image { width: 100%; height: 100%; }
.line-info { display: flex; min-width: 0; flex: 1; flex-direction: column; gap: 4px; }
.line-name { overflow: hidden; color: #454038; font-size: 10px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
.line-selections { overflow: hidden; color: #a39c91; font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.line-price { color: #b9573b; font-size: 10px; font-weight: 700; }
.line-quantity { display: flex; align-items: center; gap: 7px; }
.quantity-button { display: grid; width: 23px; height: 23px; place-items: center; border-radius: 50%; background: #f3f0ea; color: #706b62; font-size: 14px; }
.quantity-button.plus { background: #bd7555; color: #fff; }
.quantity-button.disabled { opacity: .4; }
.quantity-value { min-width: 10px; color: #4c4841; font-size: 9px; text-align: center; }
.cart-dock { position: fixed; z-index: 22; right: 0; bottom: 0; left: 0; display: flex; min-height: 60px; align-items: center; gap: 12px; padding: 8px 14px calc(8px + env(safe-area-inset-bottom)); border-top: 1px solid #eee9e1; background: rgb(255 255 255 / 97%); box-shadow: 0 -7px 24px rgb(49 39 22 / 6%); }
.cart-summary { display: flex; min-width: 0; flex: 1; align-items: center; gap: 10px; }
.cart-bag { position: relative; display: grid; width: 39px; height: 39px; flex: 0 0 39px; place-items: center; border-radius: 14px; background: #fbf1e8; font-size: 19px; cursor: pointer; }
.cart-badge { position: absolute; top: -5px; right: -5px; display: grid; min-width: 16px; height: 16px; place-items: center; padding: 0 3px; border-radius: 10px; background: #b85f40; color: #fff; font-size: 8px; }
.cart-total-copy { display: flex; min-width: 0; flex-direction: column; gap: 3px; }
.cart-total-label { color: #8b857c; font-size: 8px; }
.cart-total-price { color: #b9573b; font-size: 15px; font-weight: 750; }
.checkout-action { display: flex; min-width: 110px; height: 42px; align-items: center; justify-content: center; gap: 6px; border-radius: 23px; background: #b96d4b; color: #fff; font-size: 11px; font-weight: 700; }
</style>
