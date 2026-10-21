"use strict";
const api_auth = require("./auth.js");
const api_request = require("./request.js");
async function getAddresses() {
  return api_auth.withCustomerSession(() => api_request.request("/addresses"));
}
async function getAddress(id) {
  const addresses = await getAddresses();
  const address = addresses.find((item) => item.id === id);
  if (!address)
    throw new Error("收货地址不存在或已删除");
  return address;
}
async function saveAddress(input, id) {
  return api_auth.withCustomerSession(() => id ? api_request.request(`/addresses/${id}`, "PUT", input) : api_request.request("/addresses", "POST", input));
}
async function deleteAddress(id) {
  return api_auth.withCustomerSession(() => api_request.request(`/addresses/${id}`, "DELETE"));
}
exports.deleteAddress = deleteAddress;
exports.getAddress = getAddress;
exports.getAddresses = getAddresses;
exports.saveAddress = saveAddress;
