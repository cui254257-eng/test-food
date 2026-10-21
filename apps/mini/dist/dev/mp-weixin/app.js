"use strict";
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const common_vendor = require("./common/vendor.js");
if (!Math) {
  "./pages/index/index.js";
  "./pages/profile/index.js";
  "./pages/cart/index.js";
  "./pages/checkout/index.js";
  "./subpackages/menu/pages/products/index.js";
  "./subpackages/menu/pages/product/detail.js";
  "./subpackages/account/pages/login/index.js";
  "./subpackages/account/pages/profile/edit.js";
  "./subpackages/account/pages/address/index.js";
  "./subpackages/account/pages/address/edit.js";
}
const _sfc_main = {
  onLaunch() {
    console.info("Shop mini app launched");
  }
};
function createApp() {
  const app = common_vendor.createSSRApp(_sfc_main);
  return { app };
}
createApp().app.mount("#app");
exports.createApp = createApp;
