"use strict";
const api_request = require("./request.js");
async function getBanners() {
  const banners = await api_request.request("/banners");
  return banners.map((banner) => ({
    ...banner,
    image_url: api_request.assetUrl(banner.image_url),
    background_image_url: banner.background_image_url ? api_request.assetUrl(banner.background_image_url) : null
  }));
}
exports.getBanners = getBanners;
