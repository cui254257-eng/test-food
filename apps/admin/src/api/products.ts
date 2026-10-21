import type { Product, ProductInput } from '@shop/contracts'
import { apiRequest } from './client'

export function getProducts(query = '') {
  const params = new URLSearchParams()
  if (query.trim()) params.set('q', query.trim())
  const search = params.size ? `?${params.toString()}` : ''
  return apiRequest<Product[]>(`/products${search}`)
}

export function createProduct(input: ProductInput) {
  return apiRequest<Product>('/products', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateProduct(id: number, input: ProductInput) {
  return apiRequest<Product>(`/products/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
}

export function deleteProduct(id: number) {
  return apiRequest<void>(`/products/${id}`, { method: 'DELETE' })
}
