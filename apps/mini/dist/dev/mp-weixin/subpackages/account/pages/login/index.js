"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_auth = require("../../../../api/auth.js");
if (!Array) {
  const _easycom_wd_icon2 = common_vendor.resolveComponent("wd-icon");
  _easycom_wd_icon2();
}
const _easycom_wd_icon = () => "../../../../node-modules/@wot-ui/ui/components/wd-icon/wd-icon.js";
if (!Math) {
  _easycom_wd_icon();
}
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const loading = common_vendor.ref(false);
    async function wechatLogin() {
      if (loading.value)
        return;
      loading.value = true;
      try {
        await api_auth.loginCustomer();
        common_vendor.index.showToast({ title: "登录成功", icon: "success" });
        setTimeout(() => {
          if (getCurrentPages().length > 1)
            common_vendor.index.navigateBack();
          else
            common_vendor.index.switchTab({ url: "/pages/profile/index" });
        }, 300);
      } catch (error) {
        common_vendor.index.showModal({
          title: "微信登录失败",
          content: error instanceof Error ? error.message : "请稍后重试",
          showCancel: false,
          confirmColor: "#b86a49"
        });
      } finally {
        loading.value = false;
      }
    }
    return (_ctx, _cache) => {
      return {
        a: common_vendor.t(loading.value ? "正在登录…" : "微信登录"),
        b: common_vendor.p({
          name: "arrow-right",
          size: "16px",
          color: "#8b9c8a"
        }),
        c: loading.value ? 1 : "",
        d: loading.value,
        e: common_vendor.o(wechatLogin, "f1")
      };
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-b46600cf"]]);
wx.createPage(MiniProgramPage);
