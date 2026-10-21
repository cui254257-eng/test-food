import { request } from './request'

export interface MembershipConfig {
  spend_step_cents: number
  points_per_step: number
  tiers: Array<{ level: number; required_points: number; discount_rate: number }>
}

export function getMembershipConfig() {
  return request<MembershipConfig>('/membership/config')
}
