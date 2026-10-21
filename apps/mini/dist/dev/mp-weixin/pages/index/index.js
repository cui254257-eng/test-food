"use strict";
const common_vendor = require("../../common/vendor.js");
const api_banners = require("../../api/banners.js");
const api_announcements = require("../../api/announcements.js");
const api_restaurant = require("../../api/restaurant.js");
const api_cart = require("../../api/cart.js");
const api_profile = require("../../api/profile.js");
if (!Array) {
  const _easycom_wd_notice_bar2 = common_vendor.resolveComponent("wd-notice-bar");
  _easycom_wd_notice_bar2();
}
const _easycom_wd_notice_bar = () => "../../node-modules/@wot-ui/ui/components/wd-notice-bar/wd-notice-bar.js";
if (!Math) {
  _easycom_wd_notice_bar();
}
const MAX_ANNOUNCEMENTS = 5;
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const banners = common_vendor.ref([]);
    const bannerIndex = common_vendor.ref(0);
    const backgroundIndex = common_vendor.ref(0);
    const backgroundOpacities = common_vendor.ref([]);
    const backgroundTransitionTarget = common_vendor.ref(-1);
    const bannerWindowWidth = common_vendor.ref(375);
    const announcements = common_vendor.ref([]);
    const restaurant = common_vendor.ref(null);
    const customerProfile = common_vendor.ref(
      common_vendor.index.getStorageSync("customer-profile") ?? null
    );
    const loggedIn = common_vendor.ref(Boolean(common_vendor.index.getStorageSync("customer-token")));
    const pickupEnabled = common_vendor.computed(
      () => !restaurant.value || restaurant.value.takeaway_enabled
    );
    const deliveryEnabled = common_vendor.computed(
      () => !restaurant.value || restaurant.value.delivery_enabled
    );
    const hasLayeredBanners = common_vendor.computed(
      () => banners.value.some((banner) => Boolean(banner.background_image_url))
    );
    async function loadBanners() {
      try {
        banners.value = await api_banners.getBanners();
        if (bannerIndex.value >= banners.value.length)
          bannerIndex.value = 0;
        resetBannerBackgrounds(bannerIndex.value);
      } catch {
        banners.value = [];
        backgroundOpacities.value = [];
      }
    }
    async function loadRestaurant() {
      try {
        restaurant.value = await api_restaurant.getRestaurantSettings();
      } catch {
        restaurant.value = null;
      }
    }
    async function loadAnnouncements() {
      try {
        announcements.value = (await api_announcements.getAnnouncements()).slice(
          0,
          MAX_ANNOUNCEMENTS
        );
      } catch {
        announcements.value = [];
      }
    }
    function shouldScrollAnnouncement(content) {
      const availableWidth = Math.max(bannerWindowWidth.value - 60, 0);
      const estimatedTextWidth = Array.from(content).reduce((width, character) => {
        const isWideCharacter = /[\u2e80-\u9fff\uff00-\uffef]/.test(character);
        return width + (isWideCharacter ? 10 : 5.5);
      }, 0);
      return estimatedTextWidth > availableWidth;
    }
    function resetBannerBackgrounds(index) {
      backgroundIndex.value = index;
      backgroundTransitionTarget.value = -1;
      backgroundOpacities.value = banners.value.map(
        (_, itemIndex) => itemIndex === index ? 1 : 0
      );
    }
    function onBannerTransition(event) {
      const count = banners.value.length;
      const dx = event.detail.dx;
      if (count < 2 || !Number.isFinite(dx) || dx === 0)
        return;
      const direction = dx > 0 ? 1 : -1;
      const target = (backgroundIndex.value + direction + count) % count;
      const progress = Math.min(
        Math.abs(dx) / Math.max(bannerWindowWidth.value, 1) * 1.7,
        1
      );
      backgroundTransitionTarget.value = target;
      backgroundOpacities.value = banners.value.map((_, index) => {
        if (index === backgroundIndex.value)
          return 1 - progress;
        if (index === target)
          return progress;
        return 0;
      });
    }
    function onBannerAnimationFinish(event) {
      bannerIndex.value = event.detail.current;
      resetBannerBackgrounds(event.detail.current);
    }
    function goToBanner(index) {
      bannerIndex.value = index;
      resetBannerBackgrounds(index);
    }
    function refreshCustomerProfile() {
      const token = common_vendor.index.getStorageSync("customer-token");
      loggedIn.value = Boolean(token);
      if (!token) {
        customerProfile.value = null;
        return;
      }
      void api_profile.getCustomerProfile().then((profile) => {
        customerProfile.value = profile;
        loggedIn.value = true;
      }).catch(() => {
        if (!common_vendor.index.getStorageSync("customer-token")) {
          customerProfile.value = null;
          loggedIn.value = false;
        }
      });
    }
    function openProfile() {
      common_vendor.index.switchTab({ url: "/pages/profile/index" });
    }
    function openMenu(mode) {
      if (mode === "takeaway" && !pickupEnabled.value) {
        common_vendor.index.showToast({ title: "门店暂未开放自取", icon: "none" });
        return;
      }
      if (mode === "delivery" && !deliveryEnabled.value) {
        common_vendor.index.showToast({ title: "门店暂未开放外卖", icon: "none" });
        return;
      }
      common_vendor.index.setStorageSync("restaurant-fulfillment-mode", mode);
      common_vendor.index.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${mode}` });
    }
    async function refreshHome() {
      await Promise.all([loadBanners(), loadRestaurant(), loadAnnouncements()]);
      common_vendor.index.stopPullDownRefresh();
    }
    common_vendor.onShow(() => {
      api_cart.syncCartTabBadge();
      refreshCustomerProfile();
      void refreshHome();
    });
    common_vendor.onPullDownRefresh(() => {
      void refreshHome();
    });
    common_vendor.index.getSystemInfo({
      success: (info) => {
        bannerWindowWidth.value = info.windowWidth;
      }
    });
    return (_ctx, _cache) => {
      var _a, _b, _c, _d, _e, _f;
      return common_vendor.e({
        a: hasLayeredBanners.value
      }, hasLayeredBanners.value ? {
        b: common_vendor.f(banners.value, (banner, index, i0) => {
          return common_vendor.e({
            a: banner.background_image_url
          }, banner.background_image_url ? {
            b: banner.background_image_url || "",
            c: backgroundOpacities.value[index] ?? (index === bannerIndex.value ? 1 : 0),
            d: index === backgroundTransitionTarget.value ? 2 : 1
          } : {}, {
            e: `background-${banner.id}`
          });
        })
      } : {}, {
        c: banners.value.length
      }, banners.value.length ? {
        d: common_vendor.f(banners.value, (banner, k0, i0) => {
          return common_vendor.e({
            a: banner.image_url,
            b: banner.background_image_url ? "widthFix" : "aspectFill",
            c: banner.background_image_url ? 1 : "",
            d: banner.title
          }, banner.title ? {
            e: common_vendor.t(banner.title)
          } : {}, {
            f: banner.id
          });
        }),
        e: bannerIndex.value,
        f: banners.value.length > 1,
        g: common_vendor.o(onBannerTransition, "c4"),
        h: common_vendor.o(onBannerAnimationFinish, "c0")
      } : {
        i: common_vendor.t(((_a = restaurant.value) == null ? void 0 : _a.restaurant_name) || "好味小馆")
      }, {
        j: banners.value.length > 1
      }, banners.value.length > 1 ? {
        k: common_vendor.f(banners.value, (_, index, i0) => {
          return {
            a: index,
            b: index === bannerIndex.value ? 1 : "",
            c: common_vendor.o(($event) => goToBanner(index), index)
          };
        })
      } : {}, {
        l: (_b = customerProfile.value) == null ? void 0 : _b.avatar_data_url
      }, ((_c = customerProfile.value) == null ? void 0 : _c.avatar_data_url) ? {
        m: customerProfile.value.avatar_data_url
      } : {
        n: common_vendor.t(((_e = (_d = customerProfile.value) == null ? void 0 : _d.nickname) == null ? void 0 : _e.slice(0, 1)) || "微")
      }, {
        o: common_vendor.t(loggedIn.value ? ((_f = customerProfile.value) == null ? void 0 : _f.nickname) || "微信用户" : "欢迎来到好味小馆"),
        p: common_vendor.t(loggedIn.value ? "会员" : "去登录"),
        q: common_vendor.o(openProfile, "28"),
        r: common_vendor.t(pickupEnabled.value ? "Pickup" : "暂未开放"),
        s: !pickupEnabled.value ? 1 : "",
        t: common_vendor.o(($event) => openMenu("takeaway"), "4a"),
        v: deliveryEnabled.value
      }, deliveryEnabled.value ? {} : {}, {
        w: common_vendor.t(deliveryEnabled.value ? "Delivery" : "暂未开放"),
        x: !deliveryEnabled.value ? 1 : "",
        y: common_vendor.o(($event) => openMenu("delivery"), "8a"),
        z: announcements.value.length
      }, announcements.value.length ? {
        A: common_vendor.f(announcements.value, (announcement, index, i0) => {
          return common_vendor.e({
            a: shouldScrollAnnouncement(announcement.content)
          }, shouldScrollAnnouncement(announcement.content) ? {
            b: "83a5a03c-0-" + i0,
            c: common_vendor.p({
              text: announcement.content,
              type: "",
              color: "#72675c",
              ["background-color"]: "transparent",
              ["custom-class"]: "announcement-notice-bar",
              ["custom-style"]: "padding: 0; background: transparent;"
            })
          } : {
            d: common_vendor.t(announcement.content)
          }, {
            e: announcement.id,
            f: index > 0 ? 1 : ""
          });
        })
      } : {}, {
        B: restaurant.value
      }, restaurant.value ? common_vendor.e({
        C: common_vendor.t(restaurant.value.business_hours),
        D: restaurant.value.delivery_enabled
      }, restaurant.value.delivery_enabled ? {
        E: common_vendor.t(restaurant.value.delivery_radius_km),
        F: common_vendor.t(restaurant.value.delivery_minimum_amount.toFixed(2)),
        G: common_vendor.t(restaurant.value.delivery_fee.toFixed(2))
      } : {}, {
        H: restaurant.value.store_address
      }, restaurant.value.store_address ? {
        I: common_vendor.t(restaurant.value.store_address)
      } : {}) : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-83a5a03c"]]);
wx.createPage(MiniProgramPage);
