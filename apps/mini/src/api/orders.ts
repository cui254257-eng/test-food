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
  delivery_fee_cents: number
  total_cents: number
}

export function createOrder(input: CreateOrderInput) {
  return withCustomerSession(() => request<CreatedOrder>('/orders', 'POST', input))
}
