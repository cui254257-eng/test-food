"use strict";
const api_request = require("./request.js");
function getRestaurantSettings() {
  return api_request.request("/restaurant/settings");
}
exports.getRestaurantSettings = getRestaurantSettings;
