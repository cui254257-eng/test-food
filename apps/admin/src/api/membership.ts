import { apiRequest } from './client'

export interface MembershipTier {
  level: number
  required_points: number
  discount_rate: number
}

export interface MembershipConfig {
  spend_step_cents: number
  points_per_step: number
  tiers: MembershipTier[]
}

export interface AdminCustomer {
  id: number
  nickname: string
  points: number
  member_level: number
  discount_rate: number
  order_count: number
  paid_total_cents: number
  created_at: string
}

export interface AdminCustomerPage {
  customers: AdminCustomer[]
  total: number
  page: number
  page_size: number
}

export async function getMembershipConfig() {
  return apiRequest<MembershipConfig>('/admin/membership/config')
}

export async function updateMembershipConfig(config: MembershipConfig) {
  return apiRequest<MembershipConfig>('/admin/membership/config', {
    method: 'PUT',
    body: JSON.stringify(config),
  })
}

export function getAdminCustomers(filters: { q?: string; page: number; page_size: number }) {
  const params = new URLSearchParams({ page: String(filters.page), page_size: String(filters.page_size) })
  if (filters.q?.trim()) params.set('q', filters.q.trim())
  return apiRequest<AdminCustomerPage>(`/admin/customers?${params.toString()}`)
}
