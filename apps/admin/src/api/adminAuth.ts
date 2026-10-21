import { apiRequest } from './client'

export { clearAdminToken, getAdminToken, saveAdminToken } from './adminToken'

export interface AdminLoginInput {
  username: string
  password: string
}

interface AdminLoginResponse {
  token: string
}

interface AdminSessionResponse {
  authenticated: boolean
}

export function loginAdmin(input: AdminLoginInput) {
  return apiRequest<AdminLoginResponse>('/admin/login', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export async function verifyAdminSession() {
  const session = await apiRequest<AdminSessionResponse>('/admin/session')
  return session.authenticated
}
