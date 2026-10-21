import { withCustomerSession } from './auth'
import { request } from './request'

export interface Address {
  id: number
  recipient_name: string
  phone: string
  province: string
  city: string
  district: string
  detail: string
  is_default: boolean
}

export type AddressInput = Omit<Address, 'id'>

export async function getAddresses() {
  return withCustomerSession(() => request<Address[]>('/addresses'))
}

export async function getAddress(id: number) {
  const addresses = await getAddresses()
  const address = addresses.find((item) => item.id === id)
  if (!address) throw new Error('收货地址不存在或已删除')
  return address
}

export async function saveAddress(input: AddressInput, id?: number) {
  return withCustomerSession(() => id
    ? request<Address>(`/addresses/${id}`, 'PUT', input)
    : request<Address>('/addresses', 'POST', input))
}

export async function deleteAddress(id: number) {
  return withCustomerSession(() => request<void>(`/addresses/${id}`, 'DELETE'))
}
