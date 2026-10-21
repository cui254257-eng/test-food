import { apiRequest } from './client'

export interface Banner {
  id: number
  title: string
  sort_order: number
  image_url: string
  background_image_url: string | null
}

export interface UploadBannerInput {
  title: string
  sort_order: number
  image_data: string
  background_image_data?: string
}

export interface UpdateBannerInput {
  title: string
  sort_order: number
  image_data?: string
  background_image_data?: string
}

export function getBanners() {
  return apiRequest<Banner[]>('/banners')
}

export function uploadBanner(input: UploadBannerInput) {
  return apiRequest<Banner>('/banners', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateBanner(id: number, input: UpdateBannerInput) {
  return apiRequest<Banner>(`/banners/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
}

export function deleteBanner(id: number) {
  return apiRequest<void>(`/banners/${id}`, { method: 'DELETE' })
}
