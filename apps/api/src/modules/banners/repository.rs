use sqlx::MySqlPool;

use super::entity::{Banner, BannerImageRow};

#[derive(Clone)]
pub(crate) struct BannersRepository {
    pool: MySqlPool,
}

impl BannersRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self) -> Result<Vec<Banner>, sqlx::Error> {
        sqlx::query_as::<_, Banner>(
            "SELECT id, title, sort_order, CONCAT('/api/banners/', id, '/image?v=', \
             DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) AS image_url, \
             CASE WHEN background_image_base64 IS NULL THEN NULL ELSE \
             CONCAT('/api/banners/', id, '/background-image?v=', DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) \
             END AS background_image_url \
             FROM banners WHERE is_active = TRUE ORDER BY sort_order ASC, id DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn create(
        &self,
        title: &str,
        sort_order: i32,
        mime_type: &str,
        image_base64: &str,
        background_image: Option<(&str, &str)>,
    ) -> Result<Banner, sqlx::Error> {
        let result = match background_image {
            Some((background_mime_type, background_image_base64)) => sqlx::query(
                "INSERT INTO banners (title, sort_order, mime_type, image_base64, background_mime_type, background_image_base64) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(title)
            .bind(sort_order)
            .bind(mime_type)
            .bind(image_base64)
            .bind(background_mime_type)
            .bind(background_image_base64)
            .execute(&self.pool)
            .await?,
            None => sqlx::query(
                "INSERT INTO banners (title, sort_order, mime_type, image_base64) VALUES (?, ?, ?, ?)",
            )
            .bind(title)
            .bind(sort_order)
            .bind(mime_type)
            .bind(image_base64)
            .execute(&self.pool)
            .await?,
        };

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub(crate) async fn find_by_id(&self, id: i64) -> Result<Option<Banner>, sqlx::Error> {
        sqlx::query_as::<_, Banner>(
            "SELECT id, title, sort_order, CONCAT('/api/banners/', id, '/image?v=', \
             DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) AS image_url, \
             CASE WHEN background_image_base64 IS NULL THEN NULL ELSE \
             CONCAT('/api/banners/', id, '/background-image?v=', DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) \
             END AS background_image_url \
             FROM banners WHERE id = ? AND is_active = TRUE",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn update(
        &self,
        id: i64,
        title: &str,
        sort_order: i32,
        image: Option<(&str, &str)>,
        background_image: Option<(&str, &str)>,
    ) -> Result<Option<Banner>, sqlx::Error> {
        match (image, background_image) {
            (
                Some((mime_type, image_base64)),
                Some((background_mime_type, background_image_base64)),
            ) => {
                sqlx::query(
                    "UPDATE banners SET title = ?, sort_order = ?, mime_type = ?, image_base64 = ?, \
                     background_mime_type = ?, background_image_base64 = ? WHERE id = ? AND is_active = TRUE",
                )
                .bind(title)
                .bind(sort_order)
                .bind(mime_type)
                .bind(image_base64)
                .bind(background_mime_type)
                .bind(background_image_base64)
                .bind(id)
                .execute(&self.pool)
                .await?;
            }
            (Some((mime_type, image_base64)), None) => {
                sqlx::query(
                    "UPDATE banners SET title = ?, sort_order = ?, mime_type = ?, image_base64 = ? \
                     WHERE id = ? AND is_active = TRUE",
                )
                .bind(title)
                .bind(sort_order)
                .bind(mime_type)
                .bind(image_base64)
                .bind(id)
                .execute(&self.pool)
                .await?;
            }
            (None, Some((background_mime_type, background_image_base64))) => {
                sqlx::query(
                    "UPDATE banners SET title = ?, sort_order = ?, background_mime_type = ?, background_image_base64 = ? \
                     WHERE id = ? AND is_active = TRUE",
                )
                .bind(title)
                .bind(sort_order)
                .bind(background_mime_type)
                .bind(background_image_base64)
                .bind(id)
                .execute(&self.pool)
                .await?;
            }
            (None, None) => {
                sqlx::query(
                    "UPDATE banners SET title = ?, sort_order = ? WHERE id = ? AND is_active = TRUE",
                )
                .bind(title)
                .bind(sort_order)
                .bind(id)
                .execute(&self.pool)
                .await?;
            }
        }
        self.find_by_id(id).await
    }

    pub(crate) async fn find_image(&self, id: i64) -> Result<Option<BannerImageRow>, sqlx::Error> {
        sqlx::query_as::<_, BannerImageRow>(
            "SELECT mime_type, image_base64 FROM banners WHERE id = ? AND is_active = TRUE",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn find_background_image(
        &self,
        id: i64,
    ) -> Result<Option<BannerImageRow>, sqlx::Error> {
        sqlx::query_as::<_, BannerImageRow>(
            "SELECT background_mime_type AS mime_type, background_image_base64 AS image_base64 \
             FROM banners WHERE id = ? AND is_active = TRUE \
             AND background_mime_type IS NOT NULL AND background_image_base64 IS NOT NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM banners WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
