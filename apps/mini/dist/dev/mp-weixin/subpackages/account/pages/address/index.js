"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_addresses = require("../../../../api/addresses.js");
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const addresses = common_vendor.ref([]);
    const loading = common_vendor.ref(false);
    const errorMessage = common_vendor.ref("");
    const selecting = common_vendor.ref(false);
    const selectedAddressId = common_vendor.ref(0);
    common_vendor.onLoad((query) => {
      selecting.value = (query == null ? void 0 : query.select) === "1";
      selectedAddressId.value = Number(common_vendor.index.getStorageSync("restaurant-selected-address-id")) || 0;
    });
    function openEditor(id) {
      common_vendor.index.navigateTo({ url: id ? `/subpackages/account/pages/address/edit?id=${id}` : "/subpackages/account/pages/address/edit" });
    }
    function chooseAddress(address) {
      if (!selecting.value) {
        openEditor(address.id);
        return;
      }
      selectedAddressId.value = address.id;
      common_vendor.index.setStorageSync("restaurant-selected-address-id", address.id);
      common_vendor.index.navigateBack();
    }
    function maskPhone(phone) {
      return phone.length === 11 ? `${phone.slice(0, 3)}****${phone.slice(7)}` : phone;
    }
    async function loadAddresses() {
      loading.value = true;
      errorMessage.value = "";
      try {
        addresses.value = await api_addresses.getAddresses();
      } catch (error) {
        errorMessage.value = error instanceof Error ? error.message : "收货地址加载失败，请稍后重试";
      } finally {
        loading.value = false;
      }
    }
    function removeAddress(address) {
      common_vendor.index.showModal({
        title: "删除收货地址",
        content: `确定删除 ${address.recipient_name} 的收货地址吗？`,
        confirmColor: "#b86a49",
        success: async (result) => {
          if (!result.confirm)
            return;
          try {
            await api_addresses.deleteAddress(address.id);
            common_vendor.index.showToast({ title: "已删除", icon: "success" });
            await loadAddresses();
          } catch (error) {
            common_vendor.index.showToast({
              title: error instanceof Error ? error.message : "删除失败，请稍后重试",
              icon: "none"
            });
          }
        }
      });
    }
    common_vendor.onShow(() => {
      void loadAddresses();
    });
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: selecting.value
      }, selecting.value ? {} : {}, {
        b: loading.value && addresses.value.length === 0
      }, loading.value && addresses.value.length === 0 ? {} : errorMessage.value && addresses.value.length === 0 ? {
        d: common_vendor.t(errorMessage.value),
        e: common_vendor.o(loadAddresses, "6b")
      } : addresses.value.length === 0 ? {} : {
        g: common_vendor.f(addresses.value, (address, k0, i0) => {
          return common_vendor.e({
            a: common_vendor.t(address.recipient_name),
            b: common_vendor.t(maskPhone(address.phone)),
            c: address.is_default
          }, address.is_default ? {} : {}, {
            d: selecting.value && selectedAddressId.value === address.id
          }, selecting.value && selectedAddressId.value === address.id ? {} : {}, {
            e: common_vendor.t(address.province),
            f: common_vendor.t(address.city),
            g: common_vendor.t(address.district),
            h: common_vendor.t(address.detail),
            i: common_vendor.o(($event) => chooseAddress(address), address.id),
            j: common_vendor.o(($event) => openEditor(address.id), address.id),
            k: common_vendor.o(($event) => removeAddress(address), address.id),
            l: address.id
          });
        })
      }, {
        c: errorMessage.value && addresses.value.length === 0,
        f: addresses.value.length === 0,
        h: common_vendor.o(($event) => openEditor(), "66"),
        i: errorMessage.value && addresses.value.length > 0
      }, errorMessage.value && addresses.value.length > 0 ? {
        j: common_vendor.t(errorMessage.value)
      } : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-5baa93b3"]]);
wx.createPage(MiniProgramPage);
