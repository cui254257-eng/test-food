import { request } from './request'

export interface Announcement {
  id: number
  content: string
  is_active: boolean
  sort_order: number
  updated_at: string
}

export function getAnnouncements() {
  return request<Announcement[]>('/announcements')
}
