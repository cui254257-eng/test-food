import { request } from './request'

interface WeChatLoginResponse {
  token: string
}

export interface CustomerSession {
  authenticated: boolean
}

let activeLogin: Promise<string> | null = null

function getWeChatLoginCode() {
  return new Promise<string>((resolve, reject) => {
    uni.login({
      provider: 'weixin',
      success: (result) => result.code ? resolve(result.code) : reject(new Error('微信登录未返回凭证')),
      fail: () => reject(new Error('微信登录失败，请重试')),
    })
  })
}

export async function loginCustomer() {
  if (activeLogin) return activeLogin
  activeLogin = createCustomerSession()
  try {
    return await activeLogin
  } finally {
    activeLogin = null
  }
}

async function createCustomerSession() {
  const code = await getWeChatLoginCode()
  const result = await request<WeChatLoginResponse>('/auth/wechat', 'POST', { code })
  if (!result.token) throw new Error('登录服务没有返回有效凭证')
  uni.setStorageSync('customer-token', result.token)
  return result.token
}

export async function ensureCustomerSession() {
  const existingToken = uni.getStorageSync('customer-token') as string | undefined
  if (existingToken) return existingToken
  return loginCustomer()
}

export function getCustomerSession() {
  return request<CustomerSession>('/auth/session')
}

export async function logoutCustomer() {
  try {
    if (uni.getStorageSync('customer-token')) await request<void>('/auth/logout', 'POST')
  } finally {
    uni.removeStorageSync('customer-token')
    uni.removeStorageSync('customer-profile')
  }
}

export async function withCustomerSession<T>(operation: () => Promise<T>): Promise<T> {
  await ensureCustomerSession()
  try {
    return await operation()
  } catch (error) {
    // request() clears expired credentials on 401; renew once and replay the same action.
    if (uni.getStorageSync('customer-token')) throw error
    await ensureCustomerSession()
    return operation()
  }
}
