"use strict";
const common_vendor = require("../common/vendor.js");
const api_auth = require("./auth.js");
const api_request = require("./request.js");
function rememberProfile(profile) {
  common_vendor.index.setStorageSync("customer-profile", profile);
  return profile;
}
async function getCustomerProfile() {
  return api_auth.withCustomerSession(
    async () => rememberProfile(await api_request.request("/auth/profile"))
  );
}
async function updateCustomerNickname(nickname) {
  return api_auth.withCustomerSession(
    async () => rememberProfile(await api_request.request("/auth/profile", "PUT", { nickname }))
  );
}
async function uploadCustomerAvatar(filePath) {
  return api_auth.withCustomerSession(() => new Promise((resolve, reject) => {
    const token = common_vendor.index.getStorageSync("customer-token");
    common_vendor.index.uploadFile({
      url: `${api_request.API_BASE_URL}/auth/profile/avatar`,
      filePath,
      name: "avatar",
      header: token ? { Authorization: `Bearer ${token}` } : {},
      success: (response) => {
        let payload = response.data;
        if (typeof payload === "string") {
          try {
            payload = JSON.parse(payload);
          } catch {
            payload = null;
          }
        }
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(rememberProfile(payload));
          return;
        }
        if (response.statusCode === 401) {
          common_vendor.index.removeStorageSync("customer-token");
          common_vendor.index.removeStorageSync("customer-profile");
        }
        const message = typeof payload === "object" && payload !== null && "error" in payload ? String(payload.error) : `头像上传失败（${response.statusCode}）`;
        reject(new Error(message));
      },
      fail: (error) => reject(new Error(error.errMsg || "头像上传失败，请重试"))
    });
  }));
}
exports.getCustomerProfile = getCustomerProfile;
exports.updateCustomerNickname = updateCustomerNickname;
exports.uploadCustomerAvatar = uploadCustomerAvatar;
