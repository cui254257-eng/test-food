import { assetUrl, request } from './request'

export interface Banner {
  id: number
  title: string
  image_url: string
  background_image_url: string | null
}

export async function getBanners() {
  const banners = await request<Banner[]>('/banners')
  return banners.map((banner) => ({
    ...banner,
    image_url: assetUrl(banner.image_url),
    background_image_url: banner.background_image_url
      ? assetUrl(banner.background_image_url)
      : null,
  }))
}
