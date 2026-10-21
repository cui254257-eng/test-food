"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_cart = require("../../../../api/cart.js");
const api_products = require("../../../../api/products.js");
if (!Array) {
  const _easycom_wd_button2 = common_vendor.resolveComponent("wd-button");
  _easycom_wd_button2();
}
const _easycom_wd_button = () => "../../../../node-modules/@wot-ui/ui/components/wd-button/wd-button.js";
if (!Math) {
  _easycom_wd_button();
}
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "detail",
  setup(__props) {
    const product = common_vendor.ref(null);
    const selectedSpecifications = common_vendor.ref({});
    const error = common_vendor.ref("");
    const selectedPrice = common_vendor.computed(() => {
      if (!product.value)
        return 0;
      return product.value.price + (product.value.specifications ?? []).reduce((total, group) => {
        const selectedName = selectedSpecifications.value[group.name];
        const option = group.options.find((item) => item.name === selectedName);
        return total + ((option == null ? void 0 : option.price_delta) ?? 0);
      }, 0);
    });
    common_vendor.onLoad(async (query) => {
      const id = Number(query == null ? void 0 : query.id);
      if (!Number.isInteger(id) || id < 1) {
        error.value = "菜品不存在或链接已失效";
        return;
      }
      try {
        product.value = await api_products.getProduct(id);
        for (const group of product.value.specifications ?? []) {
          if (group.required && group.options.length) {
            selectedSpecifications.value[group.name] = group.options[0].name;
          }
        }
      } catch (reason) {
        error.value = reason instanceof Error ? reason.message : "菜品加载失败";
      }
    });
    function addToCart(showToast = true) {
      var _a;
      if (!product.value)
        return false;
      if (product.value.status !== "active" || product.value.stock <= 0) {
        common_vendor.index.showToast({ title: "该菜品暂不可购买", icon: "none" });
        return false;
      }
      const specifications = product.value.specifications ?? [];
      const missingRequired = specifications.find((group) => group.required && !selectedSpecifications.value[group.name]);
      if (missingRequired) {
        common_vendor.index.showToast({ title: `请选择${missingRequired.name}`, icon: "none" });
        return false;
      }
      const selections = specifications.flatMap((group) => selectedSpecifications.value[group.name] ? [{ group_name: group.name, option_name: selectedSpecifications.value[group.name] }] : []);
      const key = `${product.value.id}:${selections.map((item) => `${item.group_name}：${item.option_name}`).join("|")}`;
      const currentQuantity = ((_a = api_cart.getCartLines().find((line) => line.key === key)) == null ? void 0 : _a.quantity) ?? 0;
      if (currentQuantity + 1 > product.value.stock) {
        common_vendor.index.showToast({ title: `最多可点 ${product.value.stock} 份`, icon: "none" });
        return false;
      }
      const added = api_cart.addCartLine({
        key,
        product_id: product.value.id,
        name: product.value.name,
        image_url: product.value.image_url,
        unit_price: selectedPrice.value,
        quantity: 1,
        stock: product.value.stock,
        selections
      });
      if (!added) {
        common_vendor.index.showToast({ title: `最多可点 ${product.value.stock} 份`, icon: "none" });
        return false;
      }
      if (showToast) {
        common_vendor.index.showToast({ title: "已加入购物篮", icon: "success" });
        setTimeout(() => common_vendor.index.navigateBack(), 250);
      }
      return true;
    }
    function checkoutNow() {
      if (!addToCart(false))
        return;
      common_vendor.index.navigateTo({ url: "/pages/checkout/index" });
    }
    function selectSpecification(groupName, optionName) {
      selectedSpecifications.value = { ...selectedSpecifications.value, [groupName]: optionName };
    }
    function goBack() {
      common_vendor.index.navigateBack();
    }
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: product.value
      }, product.value ? common_vendor.e({
        b: product.value.image_url
      }, product.value.image_url ? {
        c: product.value.image_url
      } : {
        d: common_vendor.t(product.value.name.slice(0, 1))
      }, {
        e: common_vendor.t(product.value.name),
        f: common_vendor.t(product.value.category),
        g: common_vendor.t(product.value.description || "暂无描述"),
        h: common_vendor.t(selectedPrice.value.toFixed(2)),
        i: common_vendor.f(product.value.specifications ?? [], (group, k0, i0) => {
          return common_vendor.e({
            a: common_vendor.t(group.name),
            b: group.required
          }, group.required ? {} : {}, {
            c: common_vendor.f(group.options, (option, k1, i1) => {
              return common_vendor.e({
                a: common_vendor.t(option.name),
                b: option.price_delta > 0
              }, option.price_delta > 0 ? {
                c: common_vendor.t(option.price_delta.toFixed(2))
              } : {}, {
                d: option.name,
                e: selectedSpecifications.value[group.name] === option.name ? 1 : "",
                f: common_vendor.o(($event) => selectSpecification(group.name, option.name), option.name)
              });
            }),
            d: group.name
          });
        }),
        j: common_vendor.t(product.value.stock),
        k: product.value.stock <= 0 || product.value.status !== "active" ? 1 : "",
        l: common_vendor.o(addToCart, "e8"),
        m: product.value.stock <= 0 || product.value.status !== "active" ? 1 : "",
        n: common_vendor.o(checkoutNow, "54"),
        o: common_vendor.o(goBack, "bf")
      }) : common_vendor.e({
        p: common_vendor.t(error.value || "正在加载菜品…"),
        q: error.value
      }, error.value ? {
        r: common_vendor.o(goBack, "37"),
        s: common_vendor.p({
          size: "small",
          plain: true
        })
      } : {}));
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-009b685e"]]);
wx.createPage(MiniProgramPage);
