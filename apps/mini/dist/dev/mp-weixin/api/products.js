"use strict";
const api_request = require("./request.js");
function withAssetUrl(product) {
  return { ...product, image_url: product.image_url ? api_request.assetUrl(product.image_url) : null };
}
async function getActiveProducts(query = "") {
  const keyword = query.trim();
  const params = ["status=active"];
  if (keyword)
    params.push(`q=${encodeURIComponent(keyword)}`);
  const products = await api_request.request(`/products?${params.join("&")}`);
  return products.map(withAssetUrl);
}
async function getProduct(id) {
  return withAssetUrl(await api_request.request(`/products/${id}`));
}
exports.getActiveProducts = getActiveProducts;
exports.getProduct = getProduct;
