"use strict";
const common_vendor = require("../common/vendor.js");
const CART_STORAGE_KEY = "restaurant-order-cart";
function getCartLines() {
  const value = common_vendor.index.getStorageSync(CART_STORAGE_KEY);
  if (!Array.isArray(value))
    return [];
  return value.flatMap((candidate) => {
    if (!candidate || typeof candidate !== "object")
      return [];
    const line = candidate;
    const product_id = Number(line.product_id);
    const quantity = Number(line.quantity);
    const unit_price = Number(line.unit_price);
    if (!Number.isInteger(product_id) || product_id < 1 || !Number.isInteger(quantity) || quantity < 1 || !Number.isFinite(unit_price) || unit_price < 0 || typeof line.name !== "string")
      return [];
    const selections = Array.isArray(line.selections) ? line.selections.flatMap((selection) => {
      if (typeof selection === "string") {
        const separator = selection.indexOf("：");
        return separator > 0 && separator < selection.length - 1 ? [{ group_name: selection.slice(0, separator), option_name: selection.slice(separator + 1) }] : [];
      }
      if (!selection || typeof selection !== "object")
        return [];
      const selected = selection;
      return selected.group_name && selected.option_name ? [{ group_name: selected.group_name, option_name: selected.option_name }] : [];
    }) : [];
    return [{
      key: typeof line.key === "string" ? line.key : `${product_id}:${selections.map((item) => `${item.group_name}:${item.option_name}`).join("|")}`,
      product_id,
      name: line.name,
      image_url: typeof line.image_url === "string" ? line.image_url : null,
      unit_price,
      quantity,
      stock: Number.isInteger(line.stock) && Number(line.stock) >= 0 ? Number(line.stock) : 99,
      selections
    }];
  });
}
function getCartCount() {
  return getCartLines().reduce((total, line) => total + line.quantity, 0);
}
function syncCartTabBadge() {
  const count = getCartCount();
  if (count > 0) {
    common_vendor.index.setTabBarBadge({ index: 1, text: count > 99 ? "99+" : String(count) });
  } else {
    common_vendor.index.removeTabBarBadge({ index: 1 });
  }
}
function addCartLine(line) {
  if (line.quantity < 1 || line.quantity > line.stock)
    return false;
  const lines = getCartLines();
  const existing = lines.find((item) => item.key === line.key);
  if (existing && existing.quantity + line.quantity > line.stock)
    return false;
  if (existing) {
    existing.quantity += line.quantity;
    existing.stock = line.stock;
  } else
    lines.push(line);
  common_vendor.index.setStorageSync(CART_STORAGE_KEY, lines);
  syncCartTabBadge();
  return true;
}
function setCartLineQuantity(key, quantity) {
  const lines = getCartLines().map((line) => line.key === key ? { ...line, quantity: Math.min(quantity, line.stock) } : line).filter((line) => line.quantity > 0);
  common_vendor.index.setStorageSync(CART_STORAGE_KEY, lines);
  syncCartTabBadge();
  return lines;
}
function clearCart() {
  common_vendor.index.removeStorageSync(CART_STORAGE_KEY);
  syncCartTabBadge();
  return [];
}
exports.addCartLine = addCartLine;
exports.clearCart = clearCart;
exports.getCartLines = getCartLines;
exports.setCartLineQuantity = setCartLineQuantity;
exports.syncCartTabBadge = syncCartTabBadge;
