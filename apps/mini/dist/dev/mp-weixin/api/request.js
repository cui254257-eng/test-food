"use strict";
const common_vendor = require("../common/vendor.js");
var define_import_meta_env_default = {};
const API_BASE_URL = (define_import_meta_env_default.VITE_API_BASE_URL ?? "http://192.168.1.222:8000/api").replace(/\/$/, "");
const API_ORIGIN = API_BASE_URL.replace(/\/api$/, "");
function assetUrl(path) {
  return `${API_ORIGIN}${path.startsWith("/") ? path : `/${path}`}`;
}
function request(path, method = "GET", data) {
  return new Promise((resolve, reject) => {
    const token = common_vendor.index.getStorageSync("customer-token");
    common_vendor.index.request({
      url: `${API_BASE_URL}${path}`,
      method,
      data,
      header: token ? { Authorization: `Bearer ${token}` } : {},
      success: (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(response.data);
          return;
        }
        if (response.statusCode === 401) {
          common_vendor.index.removeStorageSync("customer-token");
          common_vendor.index.removeStorageSync("customer-profile");
        }
        const payload = response.data;
        reject(new Error(typeof payload === "object" && (payload == null ? void 0 : payload.error) ? payload.error : `请求失败（${response.statusCode}）`));
      },
      fail: reject
    });
  });
}
exports.API_BASE_URL = API_BASE_URL;
exports.assetUrl = assetUrl;
exports.request = request;
