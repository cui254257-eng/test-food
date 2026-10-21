const ADMIN_TOKEN_KEY = 'restaurant-admin-api-token'
export const ADMIN_AUTH_EXPIRED_EVENT = 'admin-auth-expired'

export function getAdminToken() {
  const persistentToken = localStorage.getItem(ADMIN_TOKEN_KEY)
  if (persistentToken) return persistentToken

  // Migrate tokens created by older releases, which only used sessionStorage.
  const legacyToken = sessionStorage.getItem(ADMIN_TOKEN_KEY)
  if (legacyToken) {
    localStorage.setItem(ADMIN_TOKEN_KEY, legacyToken)
    sessionStorage.removeItem(ADMIN_TOKEN_KEY)
    return legacyToken
  }
  return ''
}

export function saveAdminToken(token: string) {
  localStorage.setItem(ADMIN_TOKEN_KEY, token)
  sessionStorage.removeItem(ADMIN_TOKEN_KEY)
}

export function clearAdminToken() {
  localStorage.removeItem(ADMIN_TOKEN_KEY)
  sessionStorage.removeItem(ADMIN_TOKEN_KEY)
}
