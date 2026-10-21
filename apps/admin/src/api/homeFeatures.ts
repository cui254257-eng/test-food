import { apiRequest } from './client'

export interface HomeFeature {
  id: number
  label: string
  icon: string
  target_path: string
  sort_order: number
  is_active: boolean
}

export type HomeFeatureInput = Omit<HomeFeature, 'id'>

export function getHomeFeatures() {
  return apiRequest<HomeFeature[]>('/home-features')
}

export function createHomeFeature(input: HomeFeatureInput) {
  return apiRequest<HomeFeature>('/home-features', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateHomeFeature(id: number, input: HomeFeatureInput) {
  return apiRequest<HomeFeature>(`/home-features/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
}

export function deleteHomeFeature(id: number) {
  return apiRequest<void>(`/home-features/${id}`, { method: 'DELETE' })
}
