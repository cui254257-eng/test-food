<script setup lang="ts">
import { onShow } from "@dcloudio/uni-app";
import { computed, ref } from "vue";
import {
  clearCart,
  getCartLines,
  setCartLineQuantity,
  type CartLine,
} from "@/api/cart";
import { getAddresses, type Address } from "@/api/addresses";
import { createOrder, type CreateOrderInput } from "@/api/orders";
import { getCustomerProfile, type CustomerProfile } from "@/api/profile";
import { getMembershipConfig, type MembershipConfig } from "@/api/membership";
import {
  getRestaurantSettings,
  type RestaurantSettings,
} from "@/api/restaurant";

type FulfillmentMode = "dine_in" | "takeaway" | "delivery";

const lines = ref<CartLine[]>([]);
const restaurant = ref<RestaurantSettings | null>(null);
const deliveryAddress = ref<Address | null>(null);
const memberProfile = ref<CustomerProfile | null>(null);
const membershipConfig = ref<MembershipConfig | null>(null);
const mode = ref<FulfillmentMode>("takeaway");
const tableNumber = ref("");
const submitting = ref(false);
const requestKey = ref("");
const availableModes = computed(() => {
  if (!restaurant.value) return [];
  const modes: FulfillmentMode[] = [];
  if (restaurant.value.dine_in_enabled) modes.push("dine_in");
  if (restaurant.value.takeaway_enabled) modes.push("takeaway");
  if (restaurant.value.delivery_enabled) modes.push("delivery");
  return modes;
});
const itemCount = computed(() =>
  lines.value.reduce((total, line) => total + line.quantity, 0),
);
const subtotalCents = computed(() =>
  lines.value.reduce(
    (total, line) => total + Math.round(line.unit_price * 100) * line.quantity,
    0,
  ),
);
const memberDiscountCents = computed(() => {
  const rate = memberProfile.value?.discount_rate ?? 1;
  if (!memberProfile.value || rate >= 1) return 0;
  return lines.value.reduce((total, line) => {
    if (!line.discount_eligible) return total;
    const unitCents = Math.round(line.unit_price * 100);
    const discountedUnitCents = Math.round(unitCents * rate);
    return total + (unitCents - discountedUnitCents) * line.quantity;
  }, 0);
});
const deliveryFeeCents = computed(() =>
  mode.value === "delivery" && restaurant.value
    ? Math.round(restaurant.value.delivery_fee * 100)
    : 0,
);
const totalCents = computed(() => subtotalCents.value - memberDiscountCents.value + deliveryFeeCents.value);
const estimatedPoints = computed(() => {
  const config = membershipConfig.value;
  if (!config?.spend_step_cents) return 0;
  const scaledPoints = (subtotalCents.value - memberDiscountCents.value) * config.points_per_step;
  const wholePoints = Math.floor(scaledPoints / config.spend_step_cents);
  const remainder = scaledPoints % config.spend_step_cents;
  return wholePoints + (remainder * 100 > config.spend_step_cents * 6 ? 1 : 0);
});
const canSubmit = computed(
  () =>
    lines.value.length > 0 &&
    !submitting.value &&
    Boolean(restaurant.value?.is_open) &&
    availableModes.value.includes(mode.value),
);

const REQUEST_KEY_STORAGE = "restaurant-order-request-key";
const REQUEST_PAYLOAD_STORAGE = "restaurant-order-request-payload";
const ADDRESS_ID_STORAGE = "restaurant-selected-address-id";

function clearRequestKey() {
  requestKey.value = "";
  uni.removeStorageSync(REQUEST_KEY_STORAGE);
  uni.removeStorageSync(REQUEST_PAYLOAD_STORAGE);
}

function currentRequestKey(payload: Omit<CreateOrderInput, "request_key">) {
  const fingerprint = JSON.stringify(payload);
  const keyTimestamp = Number(requestKey.value.split("_", 1)[0]);
  // Rotate before server expiry so a retry cannot report an already-cancelled order as submitted.
  if (
    requestKey.value &&
    (!Number.isFinite(keyTimestamp) ||
      Date.now() - keyTimestamp > 14 * 60 * 1000)
  ) {
    clearRequestKey();
  }
  if (
    requestKey.value &&
    uni.getStorageSync(REQUEST_PAYLOAD_STORAGE) !== fingerprint
  )
    clearRequestKey();
  if (!requestKey.value) {
    requestKey.value = `${Date.now()}_${Math.random().toString(36).slice(2, 12)}`;
    uni.setStorageSync(REQUEST_KEY_STORAGE, requestKey.value);
    uni.setStorageSync(REQUEST_PAYLOAD_STORAGE, fingerprint);
  }
  return requestKey.value;
}

function money(cents: number) {
  return (cents / 100).toFixed(2);
}

function formatSelections(line: CartLine) {
  return line.selections
    .map((selection) => `${selection.group_name}：${selection.option_name}`)
    .join(" · ");
}

function refreshCart() {
  lines.value = getCartLines();
  requestKey.value = String(uni.getStorageSync(REQUEST_KEY_STORAGE) || "");
  const storedMode = uni.getStorageSync("restaurant-fulfillment-mode") as
    | FulfillmentMode
    | undefined;
  if (
    storedMode === "dine_in" ||
    storedMode === "takeaway" ||
    storedMode === "delivery"
  )
    mode.value = storedMode;
}

async function loadRestaurant() {
  try {
    restaurant.value = await getRestaurantSettings();
    if (
      !availableModes.value.includes(mode.value) &&
      availableModes.value.length
    )
      chooseMode(availableModes.value[0]);
    if (mode.value === "delivery") await loadDeliveryAddress();
  } catch {
    restaurant.value = null;
  }
}

async function loadMembership(requireSession = false) {
  const token = uni.getStorageSync("customer-token");
  memberProfile.value = null;
  try {
    membershipConfig.value = await getMembershipConfig();
  } catch {
    membershipConfig.value = null;
  }
  if (!token && !requireSession) return;
  try {
    memberProfile.value = await getCustomerProfile();
  } catch {
    memberProfile.value = null;
  }
}

function changeQuantity(line: CartLine, delta: number) {
  lines.value = setCartLineQuantity(line.key, line.quantity + delta);
  clearRequestKey();
}

function removeLine(line: CartLine) {
  lines.value = setCartLineQuantity(line.key, 0);
  clearRequestKey();
}

function chooseMode(nextMode: FulfillmentMode) {
  mode.value = nextMode;
  uni.setStorageSync("restaurant-fulfillment-mode", nextMode);
  clearRequestKey();
}

async function selectMode(nextMode: FulfillmentMode) {
  chooseMode(nextMode);
  if (nextMode === "delivery") await loadDeliveryAddress();
}

async function loadDeliveryAddress() {
  try {
    const addresses = await getAddresses();
    const storedId = Number(uni.getStorageSync(ADDRESS_ID_STORAGE));
    const previousId = deliveryAddress.value?.id;
    deliveryAddress.value =
      addresses.find((address) => address.id === storedId) ??
      addresses.find((address) => address.is_default) ??
      addresses[0] ??
      null;
    if (deliveryAddress.value)
      uni.setStorageSync(ADDRESS_ID_STORAGE, deliveryAddress.value.id);
    else uni.removeStorageSync(ADDRESS_ID_STORAGE);
    if (previousId && previousId !== deliveryAddress.value?.id)
      clearRequestKey();
  } catch {
    deliveryAddress.value = null;
  }
}

function openAddresses() {
  uni.navigateTo({ url: "/subpackages/account/pages/address/index?select=1" });
}

function goShopping() {
  const mode = String(uni.getStorageSync("restaurant-fulfillment-mode") || "takeaway");
  uni.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${encodeURIComponent(mode)}` });
}

async function submitOrder() {
  if (!canSubmit.value) return;
  if (!restaurant.value) {
    uni.showToast({ title: "门店信息加载中", icon: "none" });
    return;
  }
  if (!restaurant.value.is_open) {
    uni.showToast({ title: "门店暂未接单", icon: "none" });
    return;
  }
  if (mode.value === "dine_in" && !tableNumber.value.trim()) {
    uni.showToast({ title: "请填写桌号", icon: "none" });
    return;
  }
  if (mode.value === "delivery" && !deliveryAddress.value) {
    await loadDeliveryAddress();
    if (!deliveryAddress.value) {
      openAddresses();
      return;
    }
  }
  const minimumCents = Math.round(
    (restaurant.value.delivery_minimum_amount || 0) * 100,
  );
  if (mode.value === "delivery" && subtotalCents.value < minimumCents) {
    uni.showToast({
      title: `外卖满 ¥${money(minimumCents)} 起送`,
      icon: "none",
    });
    return;
  }

  submitting.value = true;
  try {
    await loadMembership(true);
    const pointsToEarn = estimatedPoints.value;
    const input: Omit<CreateOrderInput, "request_key"> = {
      fulfillment_mode: mode.value,
      ...(mode.value === "dine_in"
        ? { table_number: tableNumber.value.trim() }
        : {}),
      ...(mode.value === "delivery" && deliveryAddress.value
        ? { address_id: deliveryAddress.value.id }
        : {}),
      items: lines.value.map((line) => ({
        product_id: line.product_id,
        quantity: line.quantity,
        selections: line.selections,
      })),
    };
    const order = await createOrder({
      ...input,
      request_key: currentRequestKey(input),
    });
    lines.value = clearCart();
    clearRequestKey();
    uni.showModal({
      title: "订单已提交",
      content: `订单号 ${order.order_number}\n会员优惠 ¥${money(order.discount_cents)} · 合计 ¥${money(order.total_cents)}\n${pointsToEarn} 积分将在确认收款后到账\n微信支付暂未接入，15 分钟后自动取消`,
      showCancel: false,
      confirmText: "知道了",
    });
  } catch (error) {
    uni.showModal({
      title: "提交失败",
      content: error instanceof Error ? error.message : "请稍后重试",
      showCancel: false,
      confirmText: "知道了",
    });
  } finally {
    submitting.value = false;
  }
}

onShow(() => {
  refreshCart();
  void loadRestaurant();
  void loadMembership();
});
</script>

<template>
  <view class="basket-page min-h-screen px-4 pb-10 pt-4">
    <view class="page-heading">
      <view class="heading-copy">
        <text class="heading-kicker">GOOD TASTE · EASY ORDER</text>
        <text class="page-title">订单结算</text>
        <text class="page-subtitle">核对用餐方式与菜品信息</text>
      </view>
      <view class="heading-mark">🍴</view>
    </view>

    <view v-if="restaurant" class="service-banner">
      <view
        class="service-indicator"
        :class="{ closed: !restaurant.is_open }"
      />
      <view class="service-copy">
        <text class="service-state" :class="{ closed: !restaurant.is_open }">{{
          restaurant.is_open ? "门店营业中" : "门店暂不接单"
        }}</text>
        <text class="service-hours"
          >营业时间 {{ restaurant.business_hours }}</text
        >
      </view>
      <text class="service-arrow">›</text>
    </view>

    <view class="mode-card">
      <view class="section-heading">
        <view>
          <text class="section-title">用餐方式</text>
          <text class="section-caption">请选择本次订单的领取方式</text>
        </view>
        <text class="section-step">01</text>
      </view>
      <view class="mode-options">
        <view
          v-if="!restaurant || restaurant.dine_in_enabled"
          class="mode-option"
          :class="{ selected: mode === 'dine_in' }"
          @tap="selectMode('dine_in')"
        >
          <text class="mode-icon">🍽</text><text class="mode-label">堂食</text
          ><text class="mode-description">店内用餐</text>
        </view>
        <view
          v-if="!restaurant || restaurant.takeaway_enabled"
          class="mode-option"
          :class="{ selected: mode === 'takeaway' }"
          @tap="selectMode('takeaway')"
        >
          <text class="mode-icon">🥡</text
          ><text class="mode-label">门店自取</text
          ><text class="mode-description">到店取餐</text>
        </view>
        <view
          v-if="restaurant?.delivery_enabled"
          class="mode-option"
          :class="{ selected: mode === 'delivery' }"
          @tap="selectMode('delivery')"
        >
          <text class="mode-icon">🛵</text
          ><text class="mode-label">外卖配送</text
          ><text class="mode-description">送到家</text>
        </view>
      </view>
      <view v-if="mode === 'dine_in'" class="destination-card table-card">
        <view class="destination-icon">桌</view>
        <view class="destination-copy">
          <text class="destination-eyebrow">堂食桌号</text>
          <input
            v-model="tableNumber"
            placeholder="请填写桌号"
            maxlength="20"
            @input="clearRequestKey"
          />
        </view>
      </view>
      <view
        v-else-if="mode === 'delivery'"
        class="destination-card delivery-hint"
        @tap="openAddresses"
      >
        <view class="destination-icon">⌖</view>
        <view v-if="deliveryAddress" class="destination-copy chosen-address">
          <text class="destination-eyebrow">配送至</text>
          <text class="destination-title"
            >{{ deliveryAddress.recipient_name }}
            <text class="destination-phone">{{
              deliveryAddress.phone
            }}</text></text
          >
          <text class="destination-address"
            >{{ deliveryAddress.province }}{{ deliveryAddress.city
            }}{{ deliveryAddress.district }}{{ deliveryAddress.detail }}</text
          >
        </view>
        <view v-else class="destination-copy">
          <text class="destination-eyebrow">配送地址</text>
          <text class="destination-title">请选择收货地址</text>
          <text class="destination-address">仅外卖订单需要填写配送地址</text>
        </view>
        <text class="hint-arrow">›</text>
      </view>
      <view
        v-else-if="mode === 'takeaway'"
        class="destination-card pickup-card"
      >
        <view class="destination-icon pickup-icon">⌖</view>
        <view class="destination-copy">
          <text class="destination-eyebrow">门店地址</text>
          <text class="destination-title">{{
            restaurant?.restaurant_name || "门店地址"
          }}</text>
          <text class="destination-address">{{
            restaurant?.store_address || "门店暂未设置地址，请联系商家确认"
          }}</text>
          <text class="destination-footnote">自取点 · 无配送费</text>
        </view>
      </view>
      <view v-if="mode === 'delivery' && restaurant" class="delivery-rules">
        <text>配送范围 {{ restaurant.delivery_radius_km }} 公里</text>
        <text>¥{{ restaurant.delivery_minimum_amount.toFixed(2) }} 起送</text>
        <text>配送费 ¥{{ restaurant.delivery_fee.toFixed(2) }}</text>
      </view>
    </view>

    <view v-if="lines.length" class="dish-list">
      <view class="list-heading">
        <view>
          <text class="section-title">点餐清单</text>
          <text class="section-caption">美味已经为你准备好了</text>
        </view>
        <text class="list-count">{{ itemCount }} 份</text>
      </view>
      <view v-for="line in lines" :key="line.key" class="dish-card">
        <image
          v-if="line.image_url"
          class="dish-image"
          :src="line.image_url"
          mode="aspectFill"
        />
        <view v-else class="dish-placeholder">{{ line.name.slice(0, 1) }}</view>
        <view class="dish-main">
          <text class="dish-name">{{ line.name }}</text>
          <text v-if="line.selections.length" class="dish-selections">{{
            formatSelections(line)
          }}</text>
          <text class="dish-price">¥{{ line.unit_price.toFixed(2) }}</text>
          <text v-if="line.stock === 0" class="stock-warning"
            >已售罄，请移除</text
          >
        </view>
        <view class="quantity-control">
          <text class="quantity-button" @tap="changeQuantity(line, -1)">−</text>
          <text class="quantity-value">{{ line.quantity }}</text>
          <text
            class="quantity-button plus"
            :class="{ disabled: line.quantity >= line.stock }"
            @tap="changeQuantity(line, 1)"
            >＋</text
          >
        </view>
        <text class="remove-line" @tap="removeLine(line)">移除</text>
      </view>
    </view>
    <view v-else class="empty-basket">
      <text class="empty-symbol">🥣</text>
      <text class="empty-title">点餐篮为空</text>
      <text class="empty-copy">选几道喜欢的菜，马上开始美味的一餐</text>
      <view class="browse-button" @tap="goShopping">去选菜</view>
    </view>

    <view v-if="lines.length" class="basket-total">
      <view class="summary-heading">
        <view
          ><text class="section-title">费用明细</text
          ><text class="section-caption"
            >价格以提交订单时服务端计算为准</text
          ></view
        >
        <text class="summary-count">{{ itemCount }} 件商品</text>
      </view>
      <view class="cost-lines">
        <text class="cost-label">商品金额</text
        ><text class="cost-value">¥{{ money(subtotalCents) }}</text>
      </view>
      <view v-if="memberDiscountCents > 0" class="cost-lines member-discount-line">
        <text class="cost-label">V{{ memberProfile?.member_level }} 会员优惠</text
        ><text class="cost-value">-¥{{ money(memberDiscountCents) }}</text>
      </view>
      <view v-if="mode === 'delivery'" class="cost-lines">
        <text class="cost-label">配送费</text
        ><text class="cost-value">¥{{ money(deliveryFeeCents) }}</text>
      </view>
      <view v-if="membershipConfig" class="checkout-points">预计获得 {{ estimatedPoints }} 积分 · 确认收款后到账</view>
      <view class="summary-divider" />
      <view class="total-row"
        ><text class="total-caption">应付金额</text
        ><text class="total-price"
          ><text class="currency">¥</text>{{ money(totalCents) }}</text
        ></view
      >

      <wd-button
        class="submit-order"
        block
        type="primary"
        :loading="submitting"
        :disabled="!canSubmit"
        @click="submitOrder"
        >确认并提交订单</wd-button
      >
    </view>
    <view v-if="lines.length" class="continue-button" @tap="goShopping"
      >还要加菜 · 返回今日菜单</view
    >
  </view>
</template>

<style scoped>
.basket-page {
  box-sizing: border-box;
  min-height: 100vh;
  padding-bottom: calc(34px + env(safe-area-inset-bottom));
  background: #f6f4f0;
  color: #302c26;
}
.page-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 2px 2px 18px;
}
.heading-copy {
  display: flex;
  flex-direction: column;
}
.heading-kicker {
  color: #b47a5c;
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 1.7px;
}
.page-title {
  margin-top: 5px;
  color: #302c26;
  font-size: 23px;
  font-weight: 750;
  letter-spacing: 0.5px;
}
.page-subtitle {
  margin-top: 4px;
  color: #9b958b;
  font-size: 10px;
}
.heading-mark {
  display: grid;
  width: 42px;
  height: 42px;
  place-items: center;
  border: 1px solid #eee7dc;
  border-radius: 15px;
  background: #fff;
  font-size: 21px;
  box-shadow: 0 5px 16px rgb(49 39 22 / 4%);
}
.service-banner,
.mode-card,
.dish-card,
.empty-basket,
.basket-total {
  border: 1px solid rgb(231 225 215 / 70%);
  border-radius: 18px;
  background: #fff;
  box-shadow: 0 7px 24px rgb(49 39 22 / 4%);
}
.service-banner {
  display: flex;
  align-items: center;
  gap: 11px;
  margin-bottom: 12px;
  padding: 13px 15px;
}
.service-indicator,
.service-indicator.closed {
  width: 9px;
  height: 9px;
  flex: 0 0 9px;
  border-radius: 50%;
  background: #5f9b68;
  box-shadow: 0 0 0 4px #edf6ed;
}
.service-indicator.closed {
  background: #c96a4d;
  box-shadow: 0 0 0 4px #fbefeb;
}
.service-copy {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
}
.service-state {
  color: #4d8053;
  font-size: 11px;
  font-weight: 700;
}
.service-state.closed {
  color: #bd6446;
}
.service-hours {
  color: #9a948b;
  font-size: 9px;
}
.service-arrow {
  color: #c2bcb3;
  font-size: 20px;
}
.mode-card {
  padding: 16px;
}
.section-heading,
.list-heading,
.summary-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.section-heading > view,
.list-heading > view,
.summary-heading > view {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.section-title {
  color: #38342e;
  font-size: 14px;
  font-weight: 750;
}
.section-caption {
  display: block;
  color: #a39d94;
  font-size: 9px;
}
.section-step {
  color: #d6cec2;
  font-size: 18px;
  font-weight: 700;
}
.mode-options {
  display: flex;
  gap: 8px;
  margin-top: 15px;
}
.mode-option {
  position: relative;
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: center;
  flex-direction: column;
  gap: 5px;
  padding: 12px 3px 10px;
  border: 1px solid #eeeae4;
  border-radius: 14px;
  background: #fbfaf8;
  color: #79746b;
  transition:
    background 0.2s,
    border-color 0.2s;
}
.mode-option.selected {
  border-color: #ca886b;
  background: linear-gradient(150deg, #fff8f2, #fff1e7);
  color: #a45337;
  box-shadow: inset 0 0 0 1px rgb(202 136 107 / 9%);
}
.mode-icon {
  font-size: 20px;
}
.mode-label {
  color: #48433c;
  font-size: 10px;
  font-weight: 700;
}
.mode-option.selected .mode-label {
  color: #a45337;
}
.mode-description {
  color: #aaa49a;
  font-size: 8px;
}
.destination-card {
  display: flex;
  align-items: center;
  gap: 11px;
  margin-top: 12px;
  padding: 12px;
  border: 1px solid #eeeae4;
  border-radius: 13px;
  background: #fbfaf8;
}
.destination-icon {
  display: grid;
  width: 35px;
  height: 35px;
  flex: 0 0 35px;
  place-items: center;
  border-radius: 12px;
  background: #fff0e6;
  color: #b86c4b;
  font-size: 17px;
  font-weight: 700;
}
.pickup-icon {
  background: #edf4ed;
  color: #668a64;
}
.destination-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 4px;
}
.destination-eyebrow {
  color: #9d968c;
  font-size: 8px;
}
.destination-title {
  color: #49443c;
  font-size: 11px;
  font-weight: 700;
}
.destination-phone {
  margin-left: 5px;
  color: #888278;
  font-size: 9px;
  font-weight: 400;
}
.destination-address {
  overflow-wrap: anywhere;
  color: #716b61;
  font-size: 10px;
  line-height: 1.5;
}
.destination-footnote {
  color: #71906c;
  font-size: 8px;
}
.table-card input {
  width: 100%;
  color: #49443c;
  font-size: 11px;
}
.delivery-hint {
  justify-content: space-between;
}
.hint-arrow {
  padding-left: 4px;
  color: #bdb6ac;
  font-size: 21px;
}
.delivery-rules {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 9px;
}
.delivery-rules text {
  padding: 5px 8px;
  border-radius: 8px;
  background: #f7f4ef;
  color: #8b8378;
  font-size: 8px;
}
.dish-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin-top: 22px;
}
.list-heading {
  margin: 0 3px 2px;
}
.list-count,
.summary-count {
  color: #9e978d;
  font-size: 9px;
}
.dish-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 12px;
}
.dish-image,
.dish-placeholder {
  width: 62px;
  height: 62px;
  flex: 0 0 62px;
  border-radius: 13px;
}
.dish-image {
  object-fit: cover;
  background: #f6f1e9;
}
.dish-placeholder {
  display: grid;
  place-items: center;
  background: linear-gradient(145deg, #f8ebdd, #f1dcc8);
  color: #aa704d;
  font-size: 22px;
}
.dish-main {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 5px;
  padding-bottom: 8px;
}
.dish-name {
  overflow: hidden;
  color: #38352f;
  font-size: 12px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dish-selections {
  overflow: hidden;
  color: #9b958c;
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dish-price {
  color: #b9573b;
  font-size: 12px;
  font-weight: 700;
}
.stock-warning {
  color: #c45e45;
  font-size: 9px;
}
.quantity-control {
  display: flex;
  align-items: center;
  gap: 7px;
  align-self: flex-end;
  margin-bottom: 2px;
}
.quantity-button {
  display: grid;
  width: 25px;
  height: 25px;
  place-items: center;
  border-radius: 50%;
  background: #f3f0ea;
  color: #706b62;
  font-size: 15px;
}
.quantity-button.plus {
  background: #bd7555;
  color: #fff;
}
.quantity-button.disabled {
  opacity: 0.4;
}
.quantity-value {
  min-width: 12px;
  color: #4c4841;
  font-size: 10px;
  text-align: center;
}
.remove-line {
  position: absolute;
  right: 15px;
  top: 12px;
  color: #b1aba1;
  font-size: 8px;
}
.empty-basket {
  display: flex;
  align-items: center;
  flex-direction: column;
  margin-top: 13px;
  padding: 38px 20px;
  text-align: center;
}
.empty-symbol {
  font-size: 38px;
}
.empty-title {
  margin-top: 13px;
  color: #454139;
  font-size: 14px;
  font-weight: 700;
}
.empty-copy {
  max-width: 220px;
  margin-top: 7px;
  color: #9b968e;
  font-size: 10px;
  line-height: 1.6;
}
.browse-button,
.continue-button {
  margin-top: 16px;
  padding: 11px 25px;
  border-radius: 22px;
  background: #bb6d4c;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
}
.basket-total {
  display: flex;
  align-items: stretch;
  flex-direction: column;
  margin-top: 15px;
  padding: 16px;
}
.summary-heading {
  margin-bottom: 11px;
}
.cost-lines,
.total-row {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  margin-top: 9px;
}
.cost-label {
  color: #8d877e;
  font-size: 10px;
}
.cost-value {
  color: #59544c;
  font-size: 10px;
  font-weight: 600;
}
.summary-divider {
  height: 1px;
  margin-top: 13px;
  background: #f1eee9;
}
.total-row {
  margin-top: 12px;
}
.total-caption {
  color: #615c54;
  font-size: 11px;
  font-weight: 600;
}
.total-price {
  color: #b9573b;
  font-size: 21px;
  font-weight: 750;
}
.currency {
  font-size: 12px;
}
.checkout-note {
  margin-top: 8px;
  color: #aaa399;
  font-size: 8px;
  line-height: 1.5;
}
.member-discount-line .cost-label, .member-discount-line .cost-value { color: #a35e43; }
.checkout-points { margin-top: 10px; color: #9a805e; font-size: 9px; text-align: right; }
.submit-order {
  width: 100%;
  margin-top: 34px;
}
.continue-button {
  padding: 13px;
  background: transparent;
  color: #9b684f;
  text-align: center;
}
</style>
