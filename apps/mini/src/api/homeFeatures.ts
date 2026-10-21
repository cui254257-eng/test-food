import { assetUrl, request } from './request'

export interface HomeFeature {
  id: number
  label: string
  icon: string
  target_path: string
  sort_order: number
  is_active: boolean
}

export async function getHomeFeatures() {
  const features = await request<HomeFeature[]>('/home-features')
  return features
    .filter((feature) => feature.is_active)
    .map((feature) => ({
      ...feature,
      icon_url: assetUrl(`/assets/icons/${feature.icon}`),
    }))
}
