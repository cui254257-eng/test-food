export type ProductStatus = 'active' | 'inactive'

export interface Product {
  id: number
  name: string
  category: string
  description: string
  specifications: ProductSpecificationGroup[]
  price: number
  stock: number
  status: ProductStatus
  image_url: string | null
  created_at: string
  updated_at: string
}

export interface ProductSpecificationGroup {
  name: string
  required: boolean
  options: ProductSpecificationOption[]
}

export interface ProductSpecificationOption {
  name: string
  price_delta: number
}

export interface ProductInput {
  name: string
  category: string
  description: string
  specifications: ProductSpecificationGroup[]
  price: number
  stock: number
  status: ProductStatus
  image_data?: string
}
