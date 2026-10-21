"use strict";
const common_vendor = require("../../common/vendor.js");
const common_assets = require("../../common/assets.js");
const api_auth = require("../../api/auth.js");
const api_profile = require("../../api/profile.js");
if (!Array) {
  const _easycom_wd_icon2 = common_vendor.resolveComponent("wd-icon");
  _easycom_wd_icon2();
}
const _easycom_wd_icon = () => "../../node-modules/@wot-ui/ui/components/wd-icon/wd-icon.js";
if (!Math) {
  _easycom_wd_icon();
}
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "index",
  setup(__props) {
    const loggedIn = common_vendor.ref(false);
    const checking = common_vendor.ref(false);
    const loggingOut = common_vendor.ref(false);
    const profile = common_vendor.ref(
      common_vendor.index.getStorageSync("customer-profile") ?? null
    );
    function refreshSession() {
      if (!common_vendor.index.getStorageSync("customer-token")) {
        loggedIn.value = false;
        profile.value = null;
        return;
      }
      loggedIn.value = true;
      checking.value = true;
      void api_profile.getCustomerProfile().then((customerProfile) => {
        profile.value = customerProfile;
        loggedIn.value = true;
      }).catch(() => {
        const token = Boolean(common_vendor.index.getStorageSync("customer-token"));
        loggedIn.value = token;
        if (!token)
          profile.value = null;
      }).finally(() => {
        checking.value = false;
      });
    }
    function openLogin() {
      common_vendor.index.navigateTo({ url: "/subpackages/account/pages/login/index" });
    }
    function openProfileEdit() {
      common_vendor.index.navigateTo({ url: "/subpackages/account/pages/profile/edit" });
    }
    function openAddresses() {
      common_vendor.index.navigateTo({ url: "/subpackages/account/pages/address/index" });
    }
    function confirmLogout() {
      if (loggingOut.value)
        return;
      common_vendor.index.showModal({
        title: "退出登录",
        content: "确定退出当前微信账号吗？",
        confirmText: "退出登录",
        confirmColor: "#a35e43",
        success: async ({ confirm }) => {
          if (!confirm)
            return;
          loggingOut.value = true;
          try {
            await api_auth.logoutCustomer();
          } catch {
          } finally {
            loggedIn.value = false;
            profile.value = null;
            checking.value = false;
            loggingOut.value = false;
          }
          common_vendor.index.showToast({ title: "已退出登录", icon: "success" });
        }
      });
    }
    common_vendor.onShow(refreshSession);
    return (_ctx, _cache) => {
      var _a, _b, _c, _d, _e;
      return common_vendor.e({
        a: (_a = profile.value) == null ? void 0 : _a.avatar_data_url
      }, ((_b = profile.value) == null ? void 0 : _b.avatar_data_url) ? {
        b: profile.value.avatar_data_url
      } : {
        c: common_vendor.t(((_d = (_c = profile.value) == null ? void 0 : _c.nickname) == null ? void 0 : _d.slice(0, 1)) || "微")
      }, {
        d: common_vendor.t(loggedIn.value ? ((_e = profile.value) == null ? void 0 : _e.nickname) || "微信用户" : "欢迎来到好味小馆"),
        e: common_vendor.t(loggedIn.value ? checking.value ? "正在确认登录状态" : "微信快捷登录" : "登录后继续点餐"),
        f: !loggedIn.value
      }, !loggedIn.value ? {
        g: common_vendor.o(openLogin, "cf")
      } : {}, {
        h: !loggedIn.value ? 1 : "",
        i: loggedIn.value
      }, loggedIn.value ? {
        j: common_assets._imports_0,
        k: common_vendor.p({
          name: "arrow-right",
          size: "16px",
          color: "#b6aa9d"
        }),
        l: common_vendor.o(openProfileEdit, "bb")
      } : {}, {
        m: loggedIn.value
      }, loggedIn.value ? {} : {}, {
        n: common_assets._imports_1,
        o: common_vendor.p({
          name: "arrow-right",
          size: "16px",
          color: "#b6aa9d"
        }),
        p: common_vendor.o(openAddresses, "5b"),
        q: loggedIn.value
      }, loggedIn.value ? {
        r: common_vendor.t(loggingOut.value ? "正在退出…" : "退出登录"),
        s: loggingOut.value ? 1 : "",
        t: common_vendor.o(confirmLogout, "63")
      } : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-f97f9319"]]);
wx.createPage(MiniProgramPage);
