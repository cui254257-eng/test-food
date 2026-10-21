"use strict";
const api_request = require("./request.js");
const api_auth = require("./auth.js");
function createOrder(input) {
  return api_auth.withCustomerSession(() => api_request.request("/orders", "POST", input));
}
exports.createOrder = createOrder;
