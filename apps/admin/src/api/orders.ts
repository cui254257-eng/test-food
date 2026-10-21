import { apiRequest } from './client'
import { getAdminToken } from './adminAuth'

export interface AdminOrderItem {
  product_name: string
  quantity: number
  unit_price_cents: number
  line_total_cents: number
  discount_cents: number
  discount_eligible: boolean
  selected_options: Array<{ group_name: string; option_name: string }>
}

export interface AdminOrder {
  order_number: string
  customer_id: number
  customer_name: string
  fulfillment_mode: 'dine_in' | 'takeaway' | 'delivery'
  table_number: string | null
  delivery_recipient_name: string | null
  delivery_phone: string | null
  delivery_address: string | null
  status: string
  payment_status: string
  subtotal_cents: number
  discount_cents: number
  delivery_fee_cents: number
  total_cents: number
  points_earned: number
  expires_at: string
  created_at: string
  items: AdminOrderItem[]
}

export interface AdminOrderPage {
  orders: AdminOrder[]
  total: number
  page: number
  page_size: number
}

export interface AdminOrderFilters {
  q?: string
  customer_id?: number
  status?: string
  fulfillment_mode?: string
  page: number
  page_size: number
}

export function getAdminOrders(filters: AdminOrderFilters) {
  const params = new URLSearchParams()
  for (const key of ['q', 'status', 'fulfillment_mode'] as const) {
    const value = filters[key]?.trim()
    if (value) params.set(key, value)
  }
  if (filters.customer_id) params.set('customer_id', String(filters.customer_id))
  params.set('page', String(filters.page))
  params.set('page_size', String(filters.page_size))

  return apiRequest<AdminOrderPage>(`/admin/orders?${params.toString()}`, {
    headers: { Authorization: `Bearer ${getAdminToken()}` },
  })
}

export function markAdminOrderPaid(orderNumber: string) {
  return apiRequest<{ order_number: string; payment_status: string; status: string; points_earned: number }>(
    `/admin/orders/${encodeURIComponent(orderNumber)}/paid`,
    { method: 'POST' },
  )
}

export function deleteAdminOrder(orderNumber: string) {
  return apiRequest<void>(`/admin/orders/${encodeURIComponent(orderNumber)}`, {
    method: 'DELETE',
  })
}
