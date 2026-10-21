use sqlx::MySqlPool;

use super::{
    dto::AddressInput,
    entity::{Address, AddressDefaultState},
};

#[derive(Clone)]
pub(crate) struct AddressesRepository {
    pool: MySqlPool,
}

impl AddressesRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self, customer_id: i64) -> Result<Vec<Address>, sqlx::Error> {
        sqlx::query_as::<_, Address>(
            "SELECT id, recipient_name, phone, province, city, district, detail, is_default \
             FROM customer_addresses WHERE customer_id = ? \
             ORDER BY is_default DESC, id DESC",
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn create(
        &self,
        customer_id: i64,
        input: AddressInput,
    ) -> Result<Address, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        // Lock the owning customer row to serialize address changes for this user.
        // This also makes "only one default" reliable when two requests arrive together.
        let owner_exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM customers WHERE id = ? FOR UPDATE")
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?;
        if owner_exists.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }
        let address_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM customer_addresses WHERE customer_id = ?")
                .bind(customer_id)
                .fetch_one(&mut *tx)
                .await?;
        let make_default = input.is_default || address_count == 0;
        if make_default {
            sqlx::query("UPDATE customer_addresses SET is_default = FALSE WHERE customer_id = ?")
                .bind(customer_id)
                .execute(&mut *tx)
                .await?;
        }

        let result = sqlx::query(
            "INSERT INTO customer_addresses \
             (customer_id, recipient_name, phone, province, city, district, detail, is_default) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(customer_id)
        .bind(input.recipient_name.trim())
        .bind(input.phone.trim())
        .bind(input.province.trim())
        .bind(input.city.trim())
        .bind(input.district.trim())
        .bind(input.detail.trim())
        .bind(make_default)
        .execute(&mut *tx)
        .await?;
        let id = result.last_insert_id() as i64;
        tx.commit().await?;

        self.find_by_id(customer_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub(crate) async fn update(
        &self,
        customer_id: i64,
        id: i64,
        input: AddressInput,
    ) -> Result<Option<Address>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        // Use a consistent owner-first lock order in create/update/delete operations.
        let owner_exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM customers WHERE id = ? FOR UPDATE")
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?;
        if owner_exists.is_none() {
            return Ok(None);
        }
        let current = sqlx::query_as::<_, AddressDefaultState>(
            "SELECT is_default FROM customer_addresses \
             WHERE id = ? AND customer_id = ? FOR UPDATE",
        )
        .bind(id)
        .bind(customer_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(current) = current else {
            return Ok(None);
        };

        let mut make_default = input.is_default;
        if input.is_default {
            sqlx::query(
                "UPDATE customer_addresses SET is_default = FALSE \
                 WHERE customer_id = ? AND id <> ?",
            )
            .bind(customer_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        } else if current.is_default {
            // Keep one default address when possible; promote another address if this default is unset.
            let replacement: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM customer_addresses WHERE customer_id = ? AND id <> ? \
                 ORDER BY id ASC LIMIT 1 FOR UPDATE",
            )
            .bind(customer_id)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(replacement_id) = replacement {
                sqlx::query("UPDATE customer_addresses SET is_default = TRUE WHERE id = ? AND customer_id = ?")
                    .bind(replacement_id)
                    .bind(customer_id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                make_default = true;
            }
        }

        sqlx::query(
            "UPDATE customer_addresses SET recipient_name = ?, phone = ?, province = ?, \
             city = ?, district = ?, detail = ?, is_default = ? \
             WHERE id = ? AND customer_id = ?",
        )
        .bind(input.recipient_name.trim())
        .bind(input.phone.trim())
        .bind(input.province.trim())
        .bind(input.city.trim())
        .bind(input.district.trim())
        .bind(input.detail.trim())
        .bind(make_default)
        .bind(id)
        .bind(customer_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        self.find_by_id(customer_id, id).await
    }

    pub(crate) async fn delete(&self, customer_id: i64, id: i64) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        // Serialize owner-scoped mutations before inspecting/changing the default address.
        let owner_exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM customers WHERE id = ? FOR UPDATE")
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?;
        if owner_exists.is_none() {
            return Ok(false);
        }
        let current = sqlx::query_as::<_, AddressDefaultState>(
            "SELECT is_default FROM customer_addresses \
             WHERE id = ? AND customer_id = ? FOR UPDATE",
        )
        .bind(id)
        .bind(customer_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(current) = current else {
            return Ok(false);
        };

        sqlx::query("DELETE FROM customer_addresses WHERE id = ? AND customer_id = ?")
            .bind(id)
            .bind(customer_id)
            .execute(&mut *tx)
            .await?;

        if current.is_default {
            let replacement: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM customer_addresses WHERE customer_id = ? ORDER BY id ASC LIMIT 1 FOR UPDATE",
            )
            .bind(customer_id)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(replacement_id) = replacement {
                sqlx::query("UPDATE customer_addresses SET is_default = TRUE WHERE id = ? AND customer_id = ?")
                    .bind(replacement_id)
                    .bind(customer_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(true)
    }

    async fn find_by_id(&self, customer_id: i64, id: i64) -> Result<Option<Address>, sqlx::Error> {
        sqlx::query_as::<_, Address>(
            "SELECT id, recipient_name, phone, province, city, district, detail, is_default \
             FROM customer_addresses WHERE customer_id = ? AND id = ?",
        )
        .bind(customer_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
