import type { Product } from '@shop/contracts'
import { assetUrl, request } from './request'

function withAssetUrl(product: Product): Product {
  return { ...product, image_url: product.image_url ? assetUrl(product.image_url) : null }
}

export async function getActiveProducts(query = '') {
  const keyword = query.trim()
  const params = ['status=active']
  if (keyword) params.push(`q=${encodeURIComponent(keyword)}`)
  const products = await request<Product[]>(`/products?${params.join('&')}`)
  return products.map(withAssetUrl)
}

export async function getProduct(id: number) {
  return withAssetUrl(await request<Product>(`/products/${id}`))
}
