import { apiRequest } from './client'

export interface Announcement {
  id: number
  content: string
  is_active: boolean
  sort_order: number
  updated_at: string
}

export type AnnouncementInput = Pick<Announcement, 'content' | 'is_active' | 'sort_order'>

export function getAnnouncements() {
  return apiRequest<Announcement[]>('/admin/announcements')
}

export function createAnnouncement(input: AnnouncementInput) {
  return apiRequest<Announcement>('/admin/announcements', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateAnnouncement(id: number, input: AnnouncementInput) {
  return apiRequest<Announcement>(`/admin/announcements/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
}

export function deleteAnnouncement(id: number) {
  return apiRequest<void>(`/admin/announcements/${id}`, { method: 'DELETE' })
}
