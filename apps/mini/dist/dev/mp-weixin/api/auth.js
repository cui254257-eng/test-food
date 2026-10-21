"use strict";
const common_vendor = require("../common/vendor.js");
const api_request = require("./request.js");
let activeLogin = null;
function getWeChatLoginCode() {
  return new Promise((resolve, reject) => {
    common_vendor.index.login({
      provider: "weixin",
      success: (result) => result.code ? resolve(result.code) : reject(new Error("微信登录未返回凭证")),
      fail: () => reject(new Error("微信登录失败，请重试"))
    });
  });
}
async function loginCustomer() {
  if (activeLogin)
    return activeLogin;
  activeLogin = createCustomerSession();
  try {
    return await activeLogin;
  } finally {
    activeLogin = null;
  }
}
async function createCustomerSession() {
  const code = await getWeChatLoginCode();
  const result = await api_request.request("/auth/wechat", "POST", { code });
  if (!result.token)
    throw new Error("登录服务没有返回有效凭证");
  common_vendor.index.setStorageSync("customer-token", result.token);
  return result.token;
}
async function ensureCustomerSession() {
  const existingToken = common_vendor.index.getStorageSync("customer-token");
  if (existingToken)
    return existingToken;
  return loginCustomer();
}
async function logoutCustomer() {
  try {
    if (common_vendor.index.getStorageSync("customer-token"))
      await api_request.request("/auth/logout", "POST");
  } finally {
    common_vendor.index.removeStorageSync("customer-token");
    common_vendor.index.removeStorageSync("customer-profile");
  }
}
async function withCustomerSession(operation) {
  await ensureCustomerSession();
  try {
    return await operation();
  } catch (error) {
    if (common_vendor.index.getStorageSync("customer-token"))
      throw error;
    await ensureCustomerSession();
    return operation();
  }
}
exports.loginCustomer = loginCustomer;
exports.logoutCustomer = logoutCustomer;
exports.withCustomerSession = withCustomerSession;
