"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_cart = require("../../../../api/cart.js");
const api_products = require("../../../../api/products.js");
if (!Array) {
  const _easycom_wd_icon2 = common_vendor.resolveComponent("wd-icon");
  const _easycom_wd_button2 = common_vendor.resolveComponent("wd-button");
  (_easycom_wd_icon2 + _easycom_wd_button2)();
}
const _easycom_wd_icon = () => "../../../../node-modules/@wot-ui/ui/components/wd-icon/wd-icon.js";
const _easycom_wd_button = () => "../../../../node-modules/@wot-ui/ui/components/wd-button/wd-button.js";
if (!Math) {
  (_easycom_wd_icon + _easycom_wd_button)();
}
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const products = common_vendor.ref([]);
    const keyword = common_vendor.ref("");
    const sortMode = common_vendor.ref("newest");
    const selectedCategory = common_vendor.ref("全部");
    const loading = common_vendor.ref(false);
    const error = common_vendor.ref("");
    const cartLines = common_vendor.ref([]);
    const cartExpanded = common_vendor.ref(false);
    const categories = common_vendor.computed(() => [
      "全部",
      ...new Set(products.value.map((product) => product.category).filter(Boolean))
    ]);
    const sortedProducts = common_vendor.computed(() => {
      const list = products.value.filter(
        (product) => selectedCategory.value === "全部" || product.category === selectedCategory.value
      );
      if (sortMode.value === "price")
        return [...list].sort((a, b) => a.price - b.price);
      return [...list].sort((a, b) => b.id - a.id);
    });
    const cartCount = common_vendor.computed(() => cartLines.value.reduce((sum, line) => sum + line.quantity, 0));
    const cartTotalCents = common_vendor.computed(() => cartLines.value.reduce((sum, line) => sum + Math.round(line.unit_price * 100) * line.quantity, 0));
    function money(cents) {
      return (cents / 100).toFixed(2);
    }
    function refreshCart() {
      cartLines.value = api_cart.getCartLines();
      if (!cartLines.value.length)
        cartExpanded.value = false;
    }
    function updateLine(line, delta) {
      cartLines.value = api_cart.setCartLineQuantity(line.key, line.quantity + delta);
    }
    function formatSelections(line) {
      return line.selections.map((selection) => `${selection.group_name}：${selection.option_name}`).join(" · ");
    }
    function goCheckout() {
      common_vendor.index.navigateTo({ url: "/pages/checkout/index" });
    }
    async function loadProducts() {
      loading.value = true;
      error.value = "";
      try {
        products.value = await api_products.getActiveProducts(keyword.value);
      } catch (reason) {
        error.value = reason instanceof Error ? reason.message : "菜单加载失败";
      } finally {
        loading.value = false;
        common_vendor.index.stopPullDownRefresh();
      }
    }
    function openProduct(product) {
      common_vendor.index.navigateTo({ url: `/subpackages/menu/pages/product/detail?id=${product.id}` });
    }
    common_vendor.onLoad((query) => {
      keyword.value = String((query == null ? void 0 : query.q) ?? "");
      const mode = String((query == null ? void 0 : query.mode) ?? "");
      if (mode === "dine_in" || mode === "takeaway" || mode === "delivery") {
        common_vendor.index.setStorageSync("restaurant-fulfillment-mode", mode);
      }
      const sort = String((query == null ? void 0 : query.sort) ?? "");
      if (sort === "price" || sort === "newest")
        sortMode.value = sort;
      void loadProducts();
    });
    common_vendor.onPullDownRefresh(() => {
      void loadProducts();
    });
    common_vendor.onShow(refreshCart);
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: common_vendor.p({
          name: "search",
          size: "17px",
          color: "#8b8a84"
        }),
        b: common_vendor.o(($event) => loadProducts(), "bf"),
        c: keyword.value,
        d: common_vendor.o(($event) => keyword.value = $event.detail.value, "1b"),
        e: keyword.value
      }, keyword.value ? {
        f: common_vendor.o(($event) => {
          keyword.value = "";
          loadProducts();
        }, "27")
      } : {}, {
        g: common_vendor.f(categories.value, (category, k0, i0) => {
          return {
            a: common_vendor.t(category),
            b: category,
            c: selectedCategory.value === category ? 1 : "",
            d: common_vendor.o(($event) => selectedCategory.value = category, category)
          };
        }),
        h: sortMode.value === "newest" ? 1 : "",
        i: common_vendor.o(($event) => sortMode.value = "newest", "38"),
        j: sortMode.value === "price" ? 1 : "",
        k: common_vendor.o(($event) => sortMode.value = "price", "47"),
        l: loading.value && products.value.length === 0
      }, loading.value && products.value.length === 0 ? {} : error.value ? {
        n: common_vendor.t(error.value),
        o: common_vendor.o(loadProducts, "ed"),
        p: common_vendor.p({
          size: "small",
          plain: true
        })
      } : sortedProducts.value.length === 0 ? {} : {
        r: common_vendor.f(sortedProducts.value, (product, k0, i0) => {
          return common_vendor.e({
            a: product.image_url
          }, product.image_url ? {
            b: product.image_url
          } : {
            c: common_vendor.t(product.name.slice(0, 1))
          }, {
            d: common_vendor.t(product.category),
            e: common_vendor.t(product.name),
            f: common_vendor.t(product.description || "—"),
            g: common_vendor.t(product.price.toFixed(2)),
            h: product.id,
            i: common_vendor.o(($event) => openProduct(product), product.id)
          });
        })
      }, {
        m: error.value,
        q: sortedProducts.value.length === 0,
        s: cartExpanded.value
      }, cartExpanded.value ? {
        t: common_vendor.o(($event) => cartExpanded.value = false, "f3")
      } : {}, {
        v: cartExpanded.value
      }, cartExpanded.value ? {
        w: common_vendor.t(cartCount.value),
        x: common_vendor.o(($event) => cartExpanded.value = false, "94"),
        y: common_vendor.f(cartLines.value, (line, k0, i0) => {
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
            h: common_vendor.o(($event) => updateLine(line, -1), line.key),
            i: common_vendor.t(line.quantity),
            j: line.quantity >= line.stock ? 1 : "",
            k: common_vendor.o(($event) => updateLine(line, 1), line.key),
            l: line.key
          });
        })
      } : {}, {
        z: cartCount.value
      }, cartCount.value ? {
        A: common_vendor.t(cartCount.value),
        B: common_vendor.o(($event) => cartExpanded.value = !cartExpanded.value, "fd"),
        C: common_vendor.t(cartCount.value),
        D: common_vendor.t(money(cartTotalCents.value)),
        E: common_vendor.o(goCheckout, "e4")
      } : {}, {
        F: cartCount.value > 0 ? 1 : ""
      });
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-c94eb9c5"]]);
wx.createPage(MiniProgramPage);
