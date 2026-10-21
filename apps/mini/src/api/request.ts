export const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL ?? 'http://192.168.1.222:8000/api').replace(/\/$/, '')
const API_ORIGIN = API_BASE_URL.replace(/\/api$/, '')

export function assetUrl(path: string) {
  return `${API_ORIGIN}${path.startsWith('/') ? path : `/${path}`}`
}

export function request<T>(
  path: string,
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' = 'GET',
  data?: unknown,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const token = uni.getStorageSync('customer-token') as string | undefined
    uni.request({
      url: `${API_BASE_URL}${path}`,
      method,
      data: data as UniApp.RequestOptions['data'],
      header: token ? { Authorization: `Bearer ${token}` } : {},
      success: (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(response.data as T)
          return
        }
        if (response.statusCode === 401) {
          uni.removeStorageSync('customer-token')
          uni.removeStorageSync('customer-profile')
        }
        const payload = response.data as { error?: string } | string | null
        reject(new Error(typeof payload === 'object' && payload?.error ? payload.error : `请求失败（${response.statusCode}）`))
      },
      fail: reject,
    })
  })
}
