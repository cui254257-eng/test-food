use sqlx::MySqlPool;

use super::{dto::HomeFeatureInput, entity::HomeFeature};

#[derive(Clone)]
pub(crate) struct HomeFeaturesRepository {
    pool: MySqlPool,
}

impl HomeFeaturesRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self) -> Result<Vec<HomeFeature>, sqlx::Error> {
        sqlx::query_as::<_, HomeFeature>(
            "SELECT id, label, icon, target_path, sort_order, is_active \
             FROM home_features ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn create(&self, input: HomeFeatureInput) -> Result<HomeFeature, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO home_features (label, icon, target_path, sort_order, is_active) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(input.label.trim())
        .bind(input.icon.trim())
        .bind(input.target_path.trim())
        .bind(input.sort_order)
        .bind(input.is_active)
        .execute(&self.pool)
        .await?;

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub(crate) async fn update(
        &self,
        id: i64,
        input: HomeFeatureInput,
    ) -> Result<Option<HomeFeature>, sqlx::Error> {
        if self.find_by_id(id).await?.is_none() {
            return Ok(None);
        }
        sqlx::query(
            "UPDATE home_features SET label = ?, icon = ?, target_path = ?, sort_order = ?, is_active = ? \
             WHERE id = ?",
        )
        .bind(input.label.trim())
        .bind(input.icon.trim())
        .bind(input.target_path.trim())
        .bind(input.sort_order)
        .bind(input.is_active)
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.find_by_id(id).await
    }

    pub(crate) async fn find_by_id(&self, id: i64) -> Result<Option<HomeFeature>, sqlx::Error> {
        sqlx::query_as::<_, HomeFeature>(
            "SELECT id, label, icon, target_path, sort_order, is_active \
             FROM home_features WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM home_features WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
