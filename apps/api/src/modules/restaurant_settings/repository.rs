use sqlx::MySqlPool;

use super::{
    dto::{AnnouncementInput, RestaurantSettingsInput},
    entity::{Announcement, RestaurantSettings},
};

#[derive(Clone)]
pub(crate) struct RestaurantSettingsRepository {
    pool: MySqlPool,
}

impl RestaurantSettingsRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn get(&self) -> Result<RestaurantSettings, sqlx::Error> {
        sqlx::query_as::<_, RestaurantSettings>(
            "SELECT restaurant_name, business_hours, contact_phone, store_address, \
             is_open, dine_in_enabled, takeaway_enabled, delivery_enabled, delivery_radius_km, \
             delivery_minimum_amount, delivery_fee FROM restaurant_settings WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await
    }

    pub(crate) async fn update(
        &self,
        input: RestaurantSettingsInput,
    ) -> Result<RestaurantSettings, sqlx::Error> {
        sqlx::query(
            "UPDATE restaurant_settings SET restaurant_name = ?, business_hours = ?, \
             contact_phone = ?, store_address = ?, is_open = ?, dine_in_enabled = ?, takeaway_enabled = ?, \
             delivery_enabled = ?, delivery_radius_km = ?, delivery_minimum_amount = ?, delivery_fee = ? \
             WHERE id = 1",
        )
        .bind(input.restaurant_name.trim())
        .bind(input.business_hours.trim())
        .bind(input.contact_phone.trim())
        .bind(input.store_address.trim())
        .bind(input.is_open)
        .bind(input.dine_in_enabled)
        .bind(input.takeaway_enabled)
        .bind(input.delivery_enabled)
        .bind(input.delivery_radius_km)
        .bind(input.delivery_minimum_amount)
        .bind(input.delivery_fee)
        .execute(&self.pool)
        .await?;

        self.get().await
    }

    pub(crate) async fn get_announcements(&self) -> Result<Vec<Announcement>, sqlx::Error> {
        sqlx::query_as::<_, Announcement>(
            "SELECT id, content, is_active, sort_order, \
             DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at \
             FROM announcements WHERE is_active = TRUE ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn get_all_announcements(&self) -> Result<Vec<Announcement>, sqlx::Error> {
        sqlx::query_as::<_, Announcement>(
            "SELECT id, content, is_active, sort_order, \
             DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at \
             FROM announcements ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn create_announcement(
        &self,
        input: AnnouncementInput,
    ) -> Result<Announcement, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO announcements (content, is_active, sort_order) VALUES (?, ?, ?)",
        )
        .bind(input.content.trim())
        .bind(input.is_active)
        .bind(input.sort_order)
        .execute(&self.pool)
        .await?;

        self.find_announcement(result.last_insert_id() as i64)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub(crate) async fn update_announcement(
        &self,
        id: i64,
        input: AnnouncementInput,
    ) -> Result<Option<Announcement>, sqlx::Error> {
        sqlx::query(
            "UPDATE announcements SET content = ?, is_active = ?, sort_order = ? WHERE id = ?",
        )
        .bind(input.content.trim())
        .bind(input.is_active)
        .bind(input.sort_order)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.find_announcement(id).await
    }

    async fn find_announcement(&self, id: i64) -> Result<Option<Announcement>, sqlx::Error> {
        sqlx::query_as::<_, Announcement>(
            "SELECT id, content, is_active, sort_order, \
             DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at \
             FROM announcements WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn delete_announcement(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM announcements WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
