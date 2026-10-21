<script setup lang="ts">
import { onPullDownRefresh, onShow } from "@dcloudio/uni-app";
import { computed, ref } from "vue";
import { getBanners, type Banner } from "@/api/banners";
import { getAnnouncements, type Announcement } from "@/api/announcements";
import {
  getRestaurantSettings,
  type RestaurantSettings,
} from "@/api/restaurant";
import { syncCartTabBadge } from "@/api/cart";
import { getCustomerProfile, type CustomerProfile } from "@/api/profile";

const MAX_ANNOUNCEMENTS = 5;
const banners = ref<Banner[]>([]);
const bannerIndex = ref(0);
const backgroundIndex = ref(0);
const backgroundOpacities = ref<number[]>([]);
const backgroundTransitionTarget = ref(-1);
const bannerWindowWidth = ref(375);
const announcements = ref<Announcement[]>([]);
const restaurant = ref<RestaurantSettings | null>(null);
const customerProfile = ref<CustomerProfile | null>(
  (uni.getStorageSync("customer-profile") as CustomerProfile | undefined) ??
    null,
);
const loggedIn = ref(Boolean(uni.getStorageSync("customer-token")));
const pickupEnabled = computed(
  () => !restaurant.value || restaurant.value.takeaway_enabled,
);
const deliveryEnabled = computed(
  () => !restaurant.value || restaurant.value.delivery_enabled,
);
const hasLayeredBanners = computed(() =>
  banners.value.some((banner) => Boolean(banner.background_image_url)),
);

async function loadBanners() {
  try {
    banners.value = await getBanners();
    if (bannerIndex.value >= banners.value.length) bannerIndex.value = 0;
    resetBannerBackgrounds(bannerIndex.value);
  } catch {
    banners.value = [];
    backgroundOpacities.value = [];
  }
}

async function loadRestaurant() {
  try {
    restaurant.value = await getRestaurantSettings();
  } catch {
    restaurant.value = null;
  }
}

async function loadAnnouncements() {
  try {
    announcements.value = (await getAnnouncements()).slice(
      0,
      MAX_ANNOUNCEMENTS,
    );
  } catch {
    announcements.value = [];
  }
}

function shouldScrollAnnouncement(content: string) {
  const availableWidth = Math.max(bannerWindowWidth.value - 60, 0);
  const estimatedTextWidth = Array.from(content).reduce((width, character) => {
    const isWideCharacter = /[\u2e80-\u9fff\uff00-\uffef]/.test(character);
    return width + (isWideCharacter ? 10 : 5.5);
  }, 0);

  return estimatedTextWidth > availableWidth;
}

function resetBannerBackgrounds(index: number) {
  backgroundIndex.value = index;
  backgroundTransitionTarget.value = -1;
  backgroundOpacities.value = banners.value.map((_, itemIndex) =>
    itemIndex === index ? 1 : 0,
  );
}

function onBannerTransition(event: { detail: { dx: number } }) {
  const count = banners.value.length;
  const dx = event.detail.dx;
  if (count < 2 || !Number.isFinite(dx) || dx === 0) return;

  const direction = dx > 0 ? 1 : -1;
  const target = (backgroundIndex.value + direction + count) % count;
  const progress = Math.min(
    (Math.abs(dx) / Math.max(bannerWindowWidth.value, 1)) * 1.7,
    1,
  );
  backgroundTransitionTarget.value = target;
  backgroundOpacities.value = banners.value.map((_, index) => {
    if (index === backgroundIndex.value) return 1 - progress;
    if (index === target) return progress;
    return 0;
  });
}

function onBannerAnimationFinish(event: { detail: { current: number } }) {
  bannerIndex.value = event.detail.current;
  resetBannerBackgrounds(event.detail.current);
}

function goToBanner(index: number) {
  bannerIndex.value = index;
  resetBannerBackgrounds(index);
}

function refreshCustomerProfile() {
  const token = uni.getStorageSync("customer-token");
  loggedIn.value = Boolean(token);
  if (!token) {
    customerProfile.value = null;
    return;
  }

  void getCustomerProfile()
    .then((profile) => {
      customerProfile.value = profile;
      loggedIn.value = true;
    })
    .catch(() => {
      if (!uni.getStorageSync("customer-token")) {
        customerProfile.value = null;
        loggedIn.value = false;
      }
    });
}

function openProfile() {
  uni.switchTab({ url: "/pages/profile/index" });
}

function openMenu(mode: "takeaway" | "delivery") {
  if (mode === "takeaway" && !pickupEnabled.value) {
    uni.showToast({ title: "门店暂未开放自取", icon: "none" });
    return;
  }
  if (mode === "delivery" && !deliveryEnabled.value) {
    uni.showToast({ title: "门店暂未开放外卖", icon: "none" });
    return;
  }
  uni.setStorageSync("restaurant-fulfillment-mode", mode);
  uni.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${mode}` });
}

async function refreshHome() {
  await Promise.all([loadBanners(), loadRestaurant(), loadAnnouncements()]);
  uni.stopPullDownRefresh();
}

onShow(() => {
  syncCartTabBadge();
  refreshCustomerProfile();
  void refreshHome();
});
onPullDownRefresh(() => {
  void refreshHome();
});

uni.getSystemInfo({
  success: (info) => {
    bannerWindowWidth.value = info.windowWidth;
  },
});
</script>

<template>
  <view class="home-page min-h-screen pb-8">
    <view class="hero-carousel">
      <view v-if="hasLayeredBanners" class="hero-backgrounds">
        <template
          v-for="(banner, index) in banners"
          :key="`background-${banner.id}`"
        >
          <view
            v-if="banner.background_image_url"
            class="hero-background-layer"
            :style="{
              opacity:
                backgroundOpacities[index] ?? (index === bannerIndex ? 1 : 0),
              zIndex: index === backgroundTransitionTarget ? 2 : 1,
            }"
          >
            <image
              class="hero-background-image"
              :src="banner.background_image_url || ''"
              mode="aspectFill"
            />
          </view>
        </template>
      </view>
      <swiper
        v-if="banners.length"
        class="hero-swiper"
        :current="bannerIndex"
        :autoplay="banners.length > 1"
        :interval="5000"
        :duration="500"
        circular
        @transition="onBannerTransition"
        @animationfinish="onBannerAnimationFinish"
      >
        <swiper-item v-for="banner in banners" :key="banner.id">
          <view class="hero-slide">
            <view
              class="hero-image-wrap"
              :class="{ layered: banner.background_image_url }"
            >
              <image
                lazy-load
                class="hero-image"
                :src="banner.image_url"
                :mode="banner.background_image_url ? 'widthFix' : 'aspectFill'"
              />
            </view>
            <view v-if="banner.title" class="banner-copy">
              <text class="banner-title">{{ banner.title }}</text>
            </view>
          </view>
        </swiper-item>
      </swiper>
      <view v-else class="hero-fallback">
        <text class="hero-kicker">{{
          restaurant?.restaurant_name || "好味小馆"
        }}</text>
        <text class="hero-title">新鲜好味，即刻开动</text>
        <text class="hero-subtitle">自取 · 外卖 · 用心做好每一餐</text>
      </view>
      <view v-if="banners.length > 1" class="hero-pagination">
        <view
          v-for="(_, index) in banners"
          :key="index"
          class="hero-page-dot"
          :class="{ active: index === bannerIndex }"
          @tap.stop="goToBanner(index)"
        />
      </view>
    </view>

    <view class="home-sheet">
      <view class="customer-strip" @tap="openProfile">
        <view class="customer-avatar">
          <image
            v-if="customerProfile?.avatar_data_url"
            class="customer-avatar-image"
            :src="customerProfile.avatar_data_url"
            mode="aspectFill"
          />
          <text v-else class="customer-avatar-placeholder">{{
            customerProfile?.nickname?.slice(0, 1) || "微"
          }}</text>
        </view>
        <view class="customer-copy">
          <text class="customer-name">{{
            loggedIn
              ? customerProfile?.nickname || "微信用户"
              : "欢迎来到好味小馆"
          }}</text>
          <text class="customer-badge">{{ loggedIn ? "会员" : "去登录" }}</text>
        </view>
        <text class="customer-arrow">›</text>
      </view>

      <view class="sheet-divider" />

      <view class="fulfillment-options">
        <view
          class="fulfillment-option"
          hover-class="fulfillment-option-hover"
          :class="{ unavailable: !pickupEnabled }"
          @tap="openMenu('takeaway')"
        >
          <text class="fulfillment-title">自取</text>
          <text class="fulfillment-subtitle">{{
            pickupEnabled ? "Pickup" : "暂未开放"
          }}</text>
        </view>
        <view class="fulfillment-separator" />
        <view
          class="fulfillment-option"
          hover-class="fulfillment-option-hover"
          :class="{ unavailable: !deliveryEnabled }"
          @tap="openMenu('delivery')"
        >
          <text v-if="deliveryEnabled" class="fulfillment-badge">门店配送</text>
          <text class="fulfillment-title">外卖</text>
          <text class="fulfillment-subtitle">{{
            deliveryEnabled ? "Delivery" : "暂未开放"
          }}</text>
        </view>
      </view>
    </view>

    <view v-if="announcements.length" class="announcement-card mx-4 mt-5">
      <view class="announcement-label">公告</view>
      <view class="announcement-list">
        <view
          v-for="(announcement, index) in announcements"
          :key="announcement.id"
          class="announcement-item"
          :class="{ 'announcement-item-separated': index > 0 }"
        >
          <wd-notice-bar
            v-if="shouldScrollAnnouncement(announcement.content)"
            :text="announcement.content"
            type=""
            color="#72675c"
            background-color="transparent"
            custom-class="announcement-notice-bar"
            custom-style="padding: 0; background: transparent;"
          />
          <text v-else class="announcement-content">
            {{ announcement.content }}
          </text>
        </view>
      </view>
    </view>

    <view v-if="restaurant" class="store-info mx-4 mt-4">
      <view class="store-hours">
        <view class="store-info-label"><text class="store-dot" />营业时间</view>
        <text class="store-info-value">{{ restaurant.business_hours }}</text>
      </view>
      <view v-if="restaurant.delivery_enabled" class="delivery-info">
        外卖配送 {{ restaurant.delivery_radius_km }} 公里内 · 起送 ¥{{
          restaurant.delivery_minimum_amount.toFixed(2)
        }}
        · 配送费 ¥{{ restaurant.delivery_fee.toFixed(2) }}
      </view>
      <view v-if="restaurant.store_address" class="store-address">{{
        restaurant.store_address
      }}</view>
    </view>

  </view>
</template>

<style scoped>
.home-page {
  min-height: 100vh;
  background: #f7f6f2;
  color: #26241f;
}
.home-top {
  padding-top: calc(env(safe-area-inset-top) + 12px);
}
.brand-line {
  display: flex;
  align-items: center;
  gap: 9px;
}
.brand-sun {
  width: 17px;
  height: 17px;
  border: 4px solid #e4784b;
  border-radius: 50%;
  box-shadow: 0 0 0 3px #f5d5c3;
}
.top-note {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #716d64;
  font-size: 10px;
}
.top-note.closed {
  color: #b9573b;
}
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #71a16a;
}
.top-note.closed .status-dot {
  background: #cc694f;
}
.search-box {
  height: 42px;
  background: #fff;
  box-shadow: 0 3px 14px rgb(49 39 22 / 4%);
}
.clear-search {
  color: #ba6340;
  font-size: 11px;
}
.hero-carousel,
.hero-fallback {
  position: relative;
  width: 100%;
  height: 460rpx;
  overflow: hidden;
  border-radius: 25px;
  background: #f0c2a3;
  box-shadow: 0 14px 28px rgb(101 54 34 / 13%);
}
.hero-backgrounds {
  position: absolute;
  z-index: 0;
  inset: 0;
  overflow: hidden;
}
.hero-background-layer {
  position: absolute;
  inset: 0;
  overflow: hidden;
  transition: opacity 1s;
}
.hero-background-image {
  width: 100%;
  height: 100%;
}
.hero-swiper {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: inherit;
  background: transparent;
}
.hero-slide {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: inherit;
  background: transparent;
}
.hero-image-wrap {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}
.hero-image-wrap.layered {
  position: relative;
  height: 400rpx;
}
.hero-image {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
}
.hero-image-wrap.layered .hero-image {
  position: static;
  width: 100%;
  height: auto;
}
.hero-shade {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    180deg,
    rgb(35 28 25 / 4%) 38%,
    rgb(35 28 25 / 35%) 100%
  );
}
.banner-copy {
  position: absolute;
  z-index: 2;
  right: 18px;
  bottom: 39px;
  left: 18px;
}
.banner-title {
  color: #fff;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1.35;
  text-shadow: 0 1px 6px rgb(35 28 25 / 45%);
}
.hero-pagination {
  position: absolute;
  bottom: 23px;
  left: 16px;
  display: flex;
  align-items: center;
  gap: 5px;
}
.hero-page-dot {
  width: 5px;
  height: 5px;
  border-radius: 5px;
  background: rgb(255 255 255 / 55%);
  transition: width 0.18s ease;
}
.hero-page-dot.active {
  width: 16px;
  background: #fff;
}
.hero-fallback {
  display: flex;
  align-items: center;
  background: linear-gradient(118deg, #ffe7cc 0%, #f8b68d 55%, #ed8060 100%);
}
.hero-copy {
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding-left: 23px;
}
.hero-kicker {
  color: #965b3e;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.18em;
}
.hero-title {
  margin-top: 9px;
  color: #38271e;
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.04em;
}
.hero-subtitle {
  margin-top: 4px;
  color: #795743;
  font-size: 11px;
}
.hero-art {
  position: absolute;
  top: 0;
  right: 0;
  width: 52%;
  height: 100%;
  overflow: hidden;
}
.art-disc {
  position: absolute;
  top: 28px;
  right: 3px;
  width: 190px;
  height: 190px;
  border-radius: 50%;
  background: #efdfb7;
}
.art-shadow {
  position: absolute;
  right: 23px;
  bottom: 33px;
  width: 128px;
  height: 21px;
  border-radius: 50%;
  background: rgb(111 67 41 / 17%);
  filter: blur(4px);
}
.art-plate {
  position: absolute;
  right: 34px;
  bottom: 42px;
  display: grid;
  width: 142px;
  height: 142px;
  place-items: center;
  border: 3px solid #fff8e8;
  border-radius: 50%;
  background: #fffdf5;
  box-shadow: 8px 12px 0 rgb(149 91 56 / 13%);
  transform: rotate(-9deg);
}
.plate-rim {
  display: grid;
  width: 112px;
  height: 112px;
  place-items: center;
  border: 2px solid #e3cda8;
  border-radius: 50%;
  background: #f8f1dd;
}
.plate-meal {
  position: relative;
  width: 86px;
  height: 70px;
  overflow: hidden;
  border-radius: 50%;
  background: #f0dcab;
}
.meal-protein {
  position: absolute;
  top: 17px;
  left: 13px;
  width: 39px;
  height: 27px;
  border-radius: 45%;
  background: linear-gradient(135deg, #ac5d38, #d58c4c);
  transform: rotate(-15deg);
}
.meal-greens {
  position: absolute;
  top: 9px;
  right: 9px;
  width: 28px;
  height: 35px;
  border-radius: 50% 10% 50% 20%;
  background: #6c9462;
  transform: rotate(20deg);
}
.meal-rice {
  position: absolute;
  right: 9px;
  bottom: 6px;
  width: 34px;
  height: 21px;
  border-radius: 50%;
  background: #fffdf2;
}
.art-leaf {
  position: absolute;
  z-index: 1;
  width: 18px;
  height: 39px;
  border-radius: 100% 0 100% 0;
  background: #758b63;
}
.leaf-one {
  top: 33px;
  right: 26px;
  transform: rotate(21deg);
}
.leaf-two {
  top: 52px;
  right: 12px;
  height: 30px;
  background: #98a17a;
  transform: rotate(52deg);
}
.art-orbit {
  position: absolute;
  border: 1px solid rgb(255 255 255 / 58%);
  border-radius: 50%;
}
.orbit-one {
  top: 48px;
  right: 10px;
  width: 174px;
  height: 125px;
  transform: rotate(-26deg);
}
.orbit-two {
  top: 72px;
  right: 26px;
  width: 152px;
  height: 93px;
  transform: rotate(35deg);
}
.service-section {
  margin-bottom: 18px;
}
.section-heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  margin-bottom: 13px;
}
.section-heading > view {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.section-eyebrow {
  color: #c06b4c;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.15em;
}
.section-title {
  color: #302a25;
  font-size: 19px;
  font-weight: 800;
}
.section-note {
  padding-bottom: 2px;
  color: #9b958b;
  font-size: 9px;
}
.service-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 11px;
}
.service-card {
  position: relative;
  min-height: 184rpx;
  overflow: hidden;
  border-radius: 21px;
  box-shadow: 0 8px 20px rgb(64 49 34 / 7%);
}
.dine-card {
  background: linear-gradient(135deg, #fff3df 0%, #f8dfbf 100%);
}
.order-card {
  background: linear-gradient(135deg, #ffeae0 0%, #f7c8b4 100%);
}
.service-card-copy {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 5px;
  padding: 16px 14px;
}
.service-title {
  color: #382d25;
  font-size: 18px;
  font-weight: 800;
}
.service-caption {
  color: #8b7462;
  font-size: 9px;
}
.service-art {
  position: absolute;
  right: 5px;
  bottom: 0;
  width: 104px;
  height: 102px;
}
.dining-plate {
  position: absolute;
  right: 6px;
  bottom: 4px;
  display: grid;
  width: 79px;
  height: 79px;
  place-items: center;
  border: 6px solid #fffaf0;
  border-radius: 50%;
  background: #efc69b;
  box-shadow: 0 8px 14px rgb(127 81 45 / 15%);
  transform: rotate(-12deg);
}
.dining-plate view {
  width: 51px;
  height: 42px;
  border-radius: 50%;
  background:
    radial-gradient(ellipse at 32% 36%, #bf6440 0 20%, transparent 21%),
    radial-gradient(ellipse at 70% 32%, #78905d 0 18%, transparent 19%),
    radial-gradient(ellipse at 65% 72%, #fff9e8 0 25%, transparent 26%), #e6c085;
}
.dining-spark,
.order-spark {
  position: absolute;
  color: #d68156;
}
.dining-spark {
  top: 4px;
  left: 8px;
  font-size: 19px;
}
.order-cup {
  position: absolute;
  right: 22px;
  bottom: 0;
  width: 45px;
  height: 73px;
  border-radius: 8px 8px 13px 13px;
  background: linear-gradient(90deg, #fffaf1, #f5ddc7);
  box-shadow: 0 8px 14px rgb(141 69 46 / 15%);
  transform: rotate(8deg);
}
.cup-lid {
  position: absolute;
  top: -5px;
  left: -3px;
  width: 51px;
  height: 9px;
  border-radius: 5px;
  background: #bc5e45;
}
.cup-logo {
  position: absolute;
  top: 24px;
  left: 12px;
  display: grid;
  width: 21px;
  height: 21px;
  place-items: center;
  border-radius: 50%;
  background: #e99d74;
  color: #fff8ef;
  font-size: 10px;
  font-weight: 700;
}
.order-spark {
  top: 14px;
  left: 18px;
  color: #b65b44;
  font-size: 22px;
}
.service-arrow {
  position: absolute;
  z-index: 3;
  right: 12px;
  bottom: 12px;
  display: grid;
  width: 25px;
  height: 25px;
  place-items: center;
  border-radius: 50%;
  background: #bd654b;
  box-shadow: 0 4px 8px rgb(136 72 50 / 16%);
}
.announcement-card {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
  padding: 10px 14px;
  border: 1px solid #f4e7dc;
  border-radius: 14px;
  background: #fffaf5;
}
.announcement-label {
  flex: 0 0 auto;
  padding: 4px 7px;
  border-radius: 6px;
  background: #fae6db;
  color: #b95e43;
  font-size: 9px;
  font-weight: 700;
}
.announcement-list {
  display: flex;
  flex-direction: column;
}
.announcement-item {
  min-width: 0;
  padding: 5px 0;
}
.announcement-item-separated {
  border-top: 1px solid #f3ece5;
}
.announcement-content {
  display: block;
  color: #72675c;
  font-size: 10px;
  line-height: 1.45;
  white-space: normal;
}
:deep(.announcement-notice-bar) {
  min-width: 0;
  padding: 0;
  background: transparent;
  font-size: 10px;
  line-height: 1.45;
}
:deep(.announcement-notice-bar .wd-notice-bar__wrap) {
  height: 14.5px;
  line-height: 1.45;
}
.store-info {
  padding: 13px 15px;
  border: 1px solid #f2eee7;
  border-radius: 16px;
  background: #fff;
  box-shadow: 0 5px 18px rgb(49 39 22 / 4%);
}
.store-hours {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.store-info-label {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #565249;
  font-size: 11px;
}
.store-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #72a36e;
}
.store-info-value {
  color: #77736b;
  font-size: 11px;
}
.delivery-info,
.store-address {
  margin-top: 8px;
  color: #98938a;
  font-size: 10px;
  line-height: 1.5;
}
.home-page {
  /* padding-bottom: calc(24rpx + env(safe-area-inset-bottom)); */
  background: #f7f5f0;
}
.hero-carousel {
  height: 800rpx;
  border-radius: 0;
  box-shadow: none;
}
.hero-swiper {
  border-radius: 0;
}
.hero-slide {
  border-radius: 0;
}
.hero-image {
  z-index: 0;
}
.banner-copy {
  z-index: 1;
  right: 38rpx;
  bottom: 66rpx;
  left: 38rpx;
}
.banner-title {
  font-size: 30rpx;
}
.hero-pagination {
  z-index: 2;
  bottom: 34rpx;
  left: 50%;
  gap: 8rpx;
  transform: translateX(-50%);
}
.hero-page-dot {
  width: 9rpx;
  height: 9rpx;
  background: rgb(255 255 255 / 58%);
}
.hero-page-dot.active {
  width: 34rpx;
  background: #fff;
}
.hero-fallback {
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 0 52rpx;
  background:
    radial-gradient(circle at 80% 20%, #fff5df 0 16%, transparent 16.2%),
    linear-gradient(150deg, #fff0d8 0%, #f8c49d 53%, #d98560 100%);
}
.hero-kicker {
  color: #ad5e3c;
  font-size: 23rpx;
  letter-spacing: 0.2em;
}
.hero-title {
  max-width: 75%;
  margin-top: 22rpx;
  color: #523022;
  font-size: 47rpx;
  line-height: 1.22;
}
.hero-subtitle {
  margin-top: 14rpx;
  color: #875d45;
  font-size: 22rpx;
}
.home-sheet {
  position: relative;
  z-index: 3;
  margin: -26rpx 2rpx 0;
  padding: 38rpx 40rpx 32rpx;
  border-radius: 42rpx 42rpx 0 0;
  background: #fff;
  box-shadow: 0 -5rpx 24rpx rgb(72 49 34 / 3%);
}
.customer-strip {
  display: flex;
  min-height: 98rpx;
  align-items: center;
  gap: 22rpx;
}
.customer-avatar {
  display: flex;
  width: 78rpx;
  height: 78rpx;
  flex: 0 0 78rpx;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 3rpx solid #d9b99e;
  border-radius: 50%;
  background: #f7f1eb;
}
.customer-avatar-image,
.customer-avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 50%;
}
.customer-avatar-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #a87554;
  font-size: 34rpx;
  font-weight: 600;
}
.customer-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: center;
  gap: 14rpx;
}
.customer-name {
  overflow: hidden;
  color: #292622;
  font-size: 29rpx;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.customer-badge {
  flex: 0 0 auto;
  padding: 4rpx 11rpx;
  border: 1rpx solid #e2b996;
  border-radius: 18rpx;
  color: #b57a50;
  font-size: 19rpx;
  line-height: 1.2;
}
.customer-arrow {
  flex: 0 0 auto;
  color: #c4b9ae;
  font-size: 38rpx;
}
.sheet-divider {
  height: 1rpx;
  margin: 35rpx 0 17rpx;
  background: #ece9e5;
}
.fulfillment-options {
  display: flex;
  min-height: 154rpx;
  align-items: stretch;
}
.fulfillment-option {
  position: relative;
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 14rpx;
}
.fulfillment-option-hover {
  background: #fbf8f4;
}
.fulfillment-option.unavailable {
  opacity: 0.43;
}
.fulfillment-separator {
  width: 1rpx;
  align-self: center;
  height: 92rpx;
  background: #e9e6e2;
}
.fulfillment-title {
  color: #56524e;
  font-size: 40rpx;
  font-weight: 400;
  line-height: 1.3;
}
.fulfillment-subtitle {
  margin-top: 7rpx;
  color: #aaa6a2;
  font-size: 21rpx;
}
.fulfillment-badge {
  position: absolute;
  top: -2rpx;
  right: 4rpx;
  padding: 5rpx 15rpx;
  border-radius: 20rpx;
  background: #718c00;
  color: #fff;
  font-size: 19rpx;
  line-height: 1.2;
}
.announcement-card {
  margin-top: 26rpx;
}
.store-info {
  margin: 22rpx 32rpx 0;
}
</style>
