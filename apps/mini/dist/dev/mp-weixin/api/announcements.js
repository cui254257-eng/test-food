"use strict";
const api_request = require("./request.js");
function getAnnouncements() {
  return api_request.request("/announcements");
}
exports.getAnnouncements = getAnnouncements;
