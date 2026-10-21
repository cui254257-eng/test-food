"use strict";
const common_vendor = require("../../common/vendor.js");
const api_cart = require("../../api/cart.js");
const api_addresses = require("../../api/addresses.js");
const api_orders = require("../../api/orders.js");
const api_restaurant = require("../../api/restaurant.js");
if (!Array) {
  const _easycom_wd_button2 = common_vendor.resolveComponent("wd-button");
  _easycom_wd_button2();
}
const _easycom_wd_button = () => "../../node-modules/@wot-ui/ui/components/wd-button/wd-button.js";
if (!Math) {
  _easycom_wd_button();
}
const REQUEST_KEY_STORAGE = "restaurant-order-request-key";
const REQUEST_PAYLOAD_STORAGE = "restaurant-order-request-payload";
const ADDRESS_ID_STORAGE = "restaurant-selected-address-id";
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const lines = common_vendor.ref([]);
    const restaurant = common_vendor.ref(null);
    const deliveryAddress = common_vendor.ref(null);
    const mode = common_vendor.ref("takeaway");
    const tableNumber = common_vendor.ref("");
    const submitting = common_vendor.ref(false);
    const requestKey = common_vendor.ref("");
    const availableModes = common_vendor.computed(() => {
      if (!restaurant.value)
        return [];
      const modes = [];
      if (restaurant.value.dine_in_enabled)
        modes.push("dine_in");
      if (restaurant.value.takeaway_enabled)
        modes.push("takeaway");
      if (restaurant.value.delivery_enabled)
        modes.push("delivery");
      return modes;
    });
    const itemCount = common_vendor.computed(
      () => lines.value.reduce((total, line) => total + line.quantity, 0)
    );
    const subtotalCents = common_vendor.computed(
      () => lines.value.reduce(
        (total, line) => total + Math.round(line.unit_price * 100) * line.quantity,
        0
      )
    );
    const deliveryFeeCents = common_vendor.computed(
      () => mode.value === "delivery" && restaurant.value ? Math.round(restaurant.value.delivery_fee * 100) : 0
    );
    const totalCents = common_vendor.computed(() => subtotalCents.value + deliveryFeeCents.value);
    const canSubmit = common_vendor.computed(
      () => {
        var _a;
        return lines.value.length > 0 && !submitting.value && Boolean((_a = restaurant.value) == null ? void 0 : _a.is_open) && availableModes.value.includes(mode.value);
      }
    );
    function clearRequestKey() {
      requestKey.value = "";
      common_vendor.index.removeStorageSync(REQUEST_KEY_STORAGE);
      common_vendor.index.removeStorageSync(REQUEST_PAYLOAD_STORAGE);
    }
    function currentRequestKey(payload) {
      const fingerprint = JSON.stringify(payload);
      const keyTimestamp = Number(requestKey.value.split("_", 1)[0]);
      if (requestKey.value && (!Number.isFinite(keyTimestamp) || Date.now() - keyTimestamp > 14 * 60 * 1e3)) {
        clearRequestKey();
      }
      if (requestKey.value && common_vendor.index.getStorageSync(REQUEST_PAYLOAD_STORAGE) !== fingerprint)
        clearRequestKey();
      if (!requestKey.value) {
        requestKey.value = `${Date.now()}_${Math.random().toString(36).slice(2, 12)}`;
        common_vendor.index.setStorageSync(REQUEST_KEY_STORAGE, requestKey.value);
        common_vendor.index.setStorageSync(REQUEST_PAYLOAD_STORAGE, fingerprint);
      }
      return requestKey.value;
    }
    function money(cents) {
      return (cents / 100).toFixed(2);
    }
    function formatSelections(line) {
      return line.selections.map((selection) => `${selection.group_name}：${selection.option_name}`).join(" · ");
    }
    function refreshCart() {
      lines.value = api_cart.getCartLines();
      requestKey.value = String(common_vendor.index.getStorageSync(REQUEST_KEY_STORAGE) || "");
      const storedMode = common_vendor.index.getStorageSync("restaurant-fulfillment-mode");
      if (storedMode === "dine_in" || storedMode === "takeaway" || storedMode === "delivery")
        mode.value = storedMode;
    }
    async function loadRestaurant() {
      try {
        restaurant.value = await api_restaurant.getRestaurantSettings();
        if (!availableModes.value.includes(mode.value) && availableModes.value.length)
          chooseMode(availableModes.value[0]);
        if (mode.value === "delivery")
          await loadDeliveryAddress();
      } catch {
        restaurant.value = null;
      }
    }
    function changeQuantity(line, delta) {
      lines.value = api_cart.setCartLineQuantity(line.key, line.quantity + delta);
      clearRequestKey();
    }
    function removeLine(line) {
      lines.value = api_cart.setCartLineQuantity(line.key, 0);
      clearRequestKey();
    }
    function chooseMode(nextMode) {
      mode.value = nextMode;
      common_vendor.index.setStorageSync("restaurant-fulfillment-mode", nextMode);
      clearRequestKey();
    }
    async function selectMode(nextMode) {
      chooseMode(nextMode);
      if (nextMode === "delivery")
        await loadDeliveryAddress();
    }
    async function loadDeliveryAddress() {
      var _a, _b;
      try {
        const addresses = await api_addresses.getAddresses();
        const storedId = Number(common_vendor.index.getStorageSync(ADDRESS_ID_STORAGE));
        const previousId = (_a = deliveryAddress.value) == null ? void 0 : _a.id;
        deliveryAddress.value = addresses.find((address) => address.id === storedId) ?? addresses.find((address) => address.is_default) ?? addresses[0] ?? null;
        if (deliveryAddress.value)
          common_vendor.index.setStorageSync(ADDRESS_ID_STORAGE, deliveryAddress.value.id);
        else
          common_vendor.index.removeStorageSync(ADDRESS_ID_STORAGE);
        if (previousId && previousId !== ((_b = deliveryAddress.value) == null ? void 0 : _b.id))
          clearRequestKey();
      } catch {
        deliveryAddress.value = null;
      }
    }
    function openAddresses() {
      common_vendor.index.navigateTo({ url: "/subpackages/account/pages/address/index?select=1" });
    }
    function goShopping() {
      const mode2 = String(common_vendor.index.getStorageSync("restaurant-fulfillment-mode") || "takeaway");
      common_vendor.index.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${encodeURIComponent(mode2)}` });
    }
    async function submitOrder() {
      if (!canSubmit.value)
        return;
      if (!restaurant.value) {
        common_vendor.index.showToast({ title: "门店信息加载中", icon: "none" });
        return;
      }
      if (!restaurant.value.is_open) {
        common_vendor.index.showToast({ title: "门店暂未接单", icon: "none" });
        return;
      }
      if (mode.value === "dine_in" && !tableNumber.value.trim()) {
        common_vendor.index.showToast({ title: "请填写桌号", icon: "none" });
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
        (restaurant.value.delivery_minimum_amount || 0) * 100
      );
      if (mode.value === "delivery" && subtotalCents.value < minimumCents) {
        common_vendor.index.showToast({
          title: `外卖满 ¥${money(minimumCents)} 起送`,
          icon: "none"
        });
        return;
      }
      submitting.value = true;
      try {
        const input = {
          fulfillment_mode: mode.value,
          ...mode.value === "dine_in" ? { table_number: tableNumber.value.trim() } : {},
          ...mode.value === "delivery" && deliveryAddress.value ? { address_id: deliveryAddress.value.id } : {},
          items: lines.value.map((line) => ({
            product_id: line.product_id,
            quantity: line.quantity,
            selections: line.selections
          }))
        };
        const order = await api_orders.createOrder({
          ...input,
          request_key: currentRequestKey(input)
        });
        lines.value = api_cart.clearCart();
        clearRequestKey();
        common_vendor.index.showModal({
          title: "订单已提交",
          content: `订单号 ${order.order_number}
合计 ¥${money(order.total_cents)} · 待支付
微信支付暂未接入，15 分钟后自动取消`,
          showCancel: false,
          confirmText: "知道了"
        });
      } catch (error) {
        common_vendor.index.showModal({
          title: "提交失败",
          content: error instanceof Error ? error.message : "请稍后重试",
          showCancel: false,
          confirmText: "知道了"
        });
      } finally {
        submitting.value = false;
      }
    }
    common_vendor.onShow(() => {
      refreshCart();
      void loadRestaurant();
    });
    return (_ctx, _cache) => {
      var _a, _b, _c, _d;
      return common_vendor.e({
        a: restaurant.value
      }, restaurant.value ? {
        b: !restaurant.value.is_open ? 1 : "",
        c: common_vendor.t(restaurant.value.is_open ? "门店营业中" : "门店暂不接单"),
        d: !restaurant.value.is_open ? 1 : "",
        e: common_vendor.t(restaurant.value.business_hours)
      } : {}, {
        f: !restaurant.value || restaurant.value.dine_in_enabled
      }, !restaurant.value || restaurant.value.dine_in_enabled ? {
        g: mode.value === "dine_in" ? 1 : "",
        h: common_vendor.o(($event) => selectMode("dine_in"), "90")
      } : {}, {
        i: !restaurant.value || restaurant.value.takeaway_enabled
      }, !restaurant.value || restaurant.value.takeaway_enabled ? {
        j: mode.value === "takeaway" ? 1 : "",
        k: common_vendor.o(($event) => selectMode("takeaway"), "24")
      } : {}, {
        l: (_a = restaurant.value) == null ? void 0 : _a.delivery_enabled
      }, ((_b = restaurant.value) == null ? void 0 : _b.delivery_enabled) ? {
        m: mode.value === "delivery" ? 1 : "",
        n: common_vendor.o(($event) => selectMode("delivery"), "ce")
      } : {}, {
        o: mode.value === "dine_in"
      }, mode.value === "dine_in" ? {
        p: common_vendor.o([($event) => tableNumber.value = $event.detail.value, clearRequestKey], "ed"),
        q: tableNumber.value
      } : mode.value === "delivery" ? common_vendor.e({
        s: deliveryAddress.value
      }, deliveryAddress.value ? {
        t: common_vendor.t(deliveryAddress.value.recipient_name),
        v: common_vendor.t(deliveryAddress.value.phone),
        w: common_vendor.t(deliveryAddress.value.province),
        x: common_vendor.t(deliveryAddress.value.city),
        y: common_vendor.t(deliveryAddress.value.district),
        z: common_vendor.t(deliveryAddress.value.detail)
      } : {}, {
        A: common_vendor.o(openAddresses, "df")
      }) : mode.value === "takeaway" ? {
        C: common_vendor.t(((_c = restaurant.value) == null ? void 0 : _c.restaurant_name) || "门店地址"),
        D: common_vendor.t(((_d = restaurant.value) == null ? void 0 : _d.store_address) || "门店暂未设置地址，请联系商家确认")
      } : {}, {
        r: mode.value === "delivery",
        B: mode.value === "takeaway",
        E: mode.value === "delivery" && restaurant.value
      }, mode.value === "delivery" && restaurant.value ? {
        F: common_vendor.t(restaurant.value.delivery_radius_km),
        G: common_vendor.t(restaurant.value.delivery_minimum_amount.toFixed(2)),
        H: common_vendor.t(restaurant.value.delivery_fee.toFixed(2))
      } : {}, {
        I: lines.value.length
      }, lines.value.length ? {
        J: common_vendor.t(itemCount.value),
        K: common_vendor.f(lines.value, (line, k0, i0) => {
          return common_vendor.e({
            a: line.image_url
          }, line.image_url ? {
            b: line.image_url
          } : {
            c: common_vendor.t(line.name.slice(0, 1))
          }, {
            d: common_vendor.t(line.name),
            e: line.selections.length
          }, line.selections.length ? {
            f: common_vendor.t(formatSelections(line))
          } : {}, {
            g: common_vendor.t(line.unit_price.toFixed(2)),
            h: line.stock === 0
          }, line.stock === 0 ? {} : {}, {
            i: common_vendor.o(($event) => changeQuantity(line, -1), line.key),
            j: common_vendor.t(line.quantity),
            k: line.quantity >= line.stock ? 1 : "",
            l: common_vendor.o(($event) => changeQuantity(line, 1), line.key),
            m: common_vendor.o(($event) => removeLine(line), line.key),
            n: line.key
          });
        })
      } : {
        L: common_vendor.o(goShopping, "7b")
      }, {
        M: lines.value.length
      }, lines.value.length ? common_vendor.e({
        N: common_vendor.t(itemCount.value),
        O: common_vendor.t(money(subtotalCents.value)),
        P: mode.value === "delivery"
      }, mode.value === "delivery" ? {
        Q: common_vendor.t(money(deliveryFeeCents.value))
      } : {}, {
        R: common_vendor.t(money(totalCents.value)),
        S: common_vendor.o(submitOrder, "4b"),
        T: common_vendor.p({
          block: true,
          type: "primary",
          loading: submitting.value,
          disabled: !canSubmit.value
        })
      }) : {}, {
        U: lines.value.length
      }, lines.value.length ? {
        V: common_vendor.o(goShopping, "88")
      } : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-7d3d5af7"]]);
wx.createPage(MiniProgramPage);
