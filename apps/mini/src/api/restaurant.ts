import { request } from './request'

export interface RestaurantSettings {
  restaurant_name: string
  business_hours: string
  contact_phone: string
  store_address: string
  is_open: boolean
  dine_in_enabled: boolean
  takeaway_enabled: boolean
  delivery_enabled: boolean
  delivery_radius_km: number
  delivery_minimum_amount: number
  delivery_fee: number
}

export function getRestaurantSettings() {
  return request<RestaurantSettings>('/restaurant/settings')
}
