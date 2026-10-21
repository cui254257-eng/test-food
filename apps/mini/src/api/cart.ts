export interface CartLine {
  key: string
  product_id: number
  name: string
  image_url: string | null
  unit_price: number
  quantity: number
  stock: number
  selections: CartSelection[]
}

export interface CartSelection {
  group_name: string
  option_name: string
}

const CART_STORAGE_KEY = 'restaurant-order-cart'

export function getCartLines(): CartLine[] {
  const value = uni.getStorageSync(CART_STORAGE_KEY) as unknown
  if (!Array.isArray(value)) return []
  return value.flatMap((candidate: unknown) => {
    if (!candidate || typeof candidate !== 'object') return []
    const line = candidate as Partial<CartLine> & { selections?: unknown }
    const product_id = Number(line.product_id)
    const quantity = Number(line.quantity)
    const unit_price = Number(line.unit_price)
    if (!Number.isInteger(product_id) || product_id < 1 || !Number.isInteger(quantity) || quantity < 1
      || !Number.isFinite(unit_price) || unit_price < 0 || typeof line.name !== 'string') return []
    const selections = Array.isArray(line.selections)
      ? line.selections.flatMap((selection: unknown) => {
          if (typeof selection === 'string') {
            const separator = selection.indexOf('：')
            return separator > 0 && separator < selection.length - 1
              ? [{ group_name: selection.slice(0, separator), option_name: selection.slice(separator + 1) }]
              : []
          }
          if (!selection || typeof selection !== 'object') return []
          const selected = selection as Partial<CartSelection>
          return selected.group_name && selected.option_name
            ? [{ group_name: selected.group_name, option_name: selected.option_name }]
            : []
        })
      : []
    return [{
      key: typeof line.key === 'string' ? line.key : `${product_id}:${selections.map((item) => `${item.group_name}:${item.option_name}`).join('|')}`,
      product_id,
      name: line.name,
      image_url: typeof line.image_url === 'string' ? line.image_url : null,
      unit_price,
      quantity,
      stock: Number.isInteger(line.stock) && Number(line.stock) >= 0 ? Number(line.stock) : 99,
      selections,
    }]
  })
}

export function getCartCount() {
  return getCartLines().reduce((total, line) => total + line.quantity, 0)
}

export function syncCartTabBadge() {
  const count = getCartCount()
  if (count > 0) {
    uni.setTabBarBadge({ index: 1, text: count > 99 ? '99+' : String(count) })
  } else {
    uni.removeTabBarBadge({ index: 1 })
  }
}

export function addCartLine(line: CartLine) {
  if (line.quantity < 1 || line.quantity > line.stock) return false
  const lines = getCartLines()
  const existing = lines.find((item) => item.key === line.key)
  if (existing && existing.quantity + line.quantity > line.stock) return false
  if (existing) {
    existing.quantity += line.quantity
    existing.stock = line.stock
  }
  else lines.push(line)
  uni.setStorageSync(CART_STORAGE_KEY, lines)
  syncCartTabBadge()
  return true
}

export function setCartLineQuantity(key: string, quantity: number) {
  const lines = getCartLines()
    .map((line) => line.key === key ? { ...line, quantity: Math.min(quantity, line.stock) } : line)
    .filter((line) => line.quantity > 0)
  uni.setStorageSync(CART_STORAGE_KEY, lines)
  syncCartTabBadge()
  return lines
}

export function clearCart() {
  uni.removeStorageSync(CART_STORAGE_KEY)
  syncCartTabBadge()
  return [] as CartLine[]
}
