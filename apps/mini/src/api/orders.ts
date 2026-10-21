import { request } from './request'
import type { CartLine } from './cart'
import { withCustomerSession } from './auth'

export interface CreateOrderInput {
  request_key: string
  fulfillment_mode: 'dine_in' | 'takeaway' | 'delivery'
  table_number?: string
  address_id?: number
  items: Array<{
    product_id: number
    quantity: number
    selections: CartLine['selections']
  }>
}

export interface CreatedOrder {
  order_number: string
  fulfillment_mode: CreateOrderInput['fulfillment_mode']
  status: 'pending_payment'
  payment_status: 'unpaid'
  subtotal_cents: number
  discount_cents: number
  delivery_fee_cents: number
  total_cents: number
  points_earned: number
}

export interface OrderSummary {
  order_number: string
  customer_id: number
  customer_name: string
  fulfillment_mode: CreateOrderInput['fulfillment_mode']
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
  items: Array<{
    product_name: string
    quantity: number
    unit_price_cents: number
    line_total_cents: number
    discount_cents: number
    discount_eligible: boolean
    selected_options: Array<{ group_name: string; option_name: string }>
  }>
}

export function createOrder(input: CreateOrderInput) {
  return withCustomerSession(() => request<CreatedOrder>('/orders', 'POST', input))
}

export function getMyOrders() {
  return withCustomerSession(() => request<OrderSummary[]>('/orders'))
}
