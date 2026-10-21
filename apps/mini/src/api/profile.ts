import { withCustomerSession } from './auth'
import { API_BASE_URL, request } from './request'

export interface CustomerProfile {
  nickname: string
  avatar_data_url: string | null
  member_level: number
  points: number
  discount_rate: number
  next_level: number | null
  next_level_points: number | null
  points_to_next_level: number | null
}

function rememberProfile(profile: CustomerProfile) {
  uni.setStorageSync('customer-profile', profile)
  return profile
}

export async function getCustomerProfile() {
  return withCustomerSession(async () =>
    rememberProfile(await request<CustomerProfile>('/auth/profile')),
  )
}

export async function updateCustomerNickname(nickname: string) {
  return withCustomerSession(async () =>
    rememberProfile(await request<CustomerProfile>('/auth/profile', 'PUT', { nickname })),
  )
}

export async function uploadCustomerAvatar(filePath: string) {
  return withCustomerSession(() => new Promise<CustomerProfile>((resolve, reject) => {
    const token = uni.getStorageSync('customer-token') as string | undefined
    uni.uploadFile({
      url: `${API_BASE_URL}/auth/profile/avatar`,
      filePath,
      name: 'avatar',
      header: token ? { Authorization: `Bearer ${token}` } : {},
      success: (response) => {
        let payload: unknown = response.data
        if (typeof payload === 'string') {
          try {
            payload = JSON.parse(payload)
          } catch {
            payload = null
          }
        }

        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(rememberProfile(payload as CustomerProfile))
          return
        }
        if (response.statusCode === 401) {
          uni.removeStorageSync('customer-token')
          uni.removeStorageSync('customer-profile')
        }
        const message = typeof payload === 'object' && payload !== null && 'error' in payload
          ? String((payload as { error: unknown }).error)
          : `头像上传失败（${response.statusCode}）`
        reject(new Error(message))
      },
      fail: (error) => reject(new Error(error.errMsg || '头像上传失败，请重试')),
    })
  }))
}
