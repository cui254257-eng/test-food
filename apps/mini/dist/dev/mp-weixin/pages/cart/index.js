"use strict";
const common_vendor = require("../../common/vendor.js");
const api_cart = require("../../api/cart.js");
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const lines = common_vendor.ref([]);
    const itemCount = common_vendor.computed(() => lines.value.reduce((sum, line) => sum + line.quantity, 0));
    const totalCents = common_vendor.computed(() => lines.value.reduce(
      (sum, line) => sum + Math.round(line.unit_price * 100) * line.quantity,
      0
    ));
    function refreshCart() {
      lines.value = api_cart.getCartLines();
    }
    function changeQuantity(line, delta) {
      lines.value = api_cart.setCartLineQuantity(line.key, line.quantity + delta);
    }
    function removeLine(line) {
      lines.value = api_cart.setCartLineQuantity(line.key, 0);
    }
    function formatSelections(line) {
      return line.selections.map((selection) => `${selection.group_name}：${selection.option_name}`).join(" · ");
    }
    function goCheckout() {
      common_vendor.index.navigateTo({ url: "/pages/checkout/index" });
    }
    function goMenu() {
      const mode = String(common_vendor.index.getStorageSync("restaurant-fulfillment-mode") || "takeaway");
      common_vendor.index.navigateTo({ url: `/subpackages/menu/pages/products/index?mode=${encodeURIComponent(mode)}` });
    }
    common_vendor.onShow(refreshCart);
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: lines.value.length
      }, lines.value.length ? {
        b: common_vendor.t(itemCount.value),
        c: common_vendor.f(lines.value, (line, k0, i0) => {
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
            h: common_vendor.o(($event) => changeQuantity(line, -1), line.key),
            i: common_vendor.t(line.quantity),
            j: line.quantity >= line.stock ? 1 : "",
            k: common_vendor.o(($event) => changeQuantity(line, 1), line.key),
            l: common_vendor.o(($event) => removeLine(line), line.key),
            m: line.key
          });
        })
      } : {
        d: common_vendor.o(goMenu, "34")
      }, {
        e: lines.value.length
      }, lines.value.length ? {
        f: common_vendor.t(itemCount.value),
        g: common_vendor.t((totalCents.value / 100).toFixed(2)),
        h: common_vendor.t((totalCents.value / 100).toFixed(2)),
        i: common_vendor.o(goCheckout, "4d")
      } : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-3277fd7b"]]);
wx.createPage(MiniProgramPage);
