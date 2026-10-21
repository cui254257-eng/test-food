use sqlx::MySqlPool;

use super::{
    dto::ProductInput,
    dto::ProductQuery,
    entity::{Product, ProductImageRow},
};

#[derive(Clone)]
pub(crate) struct ProductsRepository {
    pool: MySqlPool,
}

impl ProductsRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self, query: ProductQuery) -> Result<Vec<Product>, sqlx::Error> {
        let keyword = query.q.filter(|value| !value.trim().is_empty());
        let status = query.status;
        sqlx::query_as::<_, Product>(
            "SELECT id, name, category, description, COALESCE(specifications, JSON_ARRAY()) AS specifications, price, stock, status, discount_eligible, \
             CASE WHEN image_base64 IS NULL THEN NULL ELSE CONCAT('/api/products/', id, '/image?v=', \
             DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) END AS image_url, \
             DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at, \
             DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at \
             FROM products \
             WHERE (? IS NULL OR status = ?) \
               AND (? IS NULL OR lower(name) LIKE lower(CONCAT('%', ?, '%'))) \
             ORDER BY id DESC",
        )
        .bind(status.clone())
        .bind(status)
        .bind(keyword.clone())
        .bind(keyword)
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn find_by_id(&self, id: i64) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, name, category, description, COALESCE(specifications, JSON_ARRAY()) AS specifications, price, stock, status, discount_eligible, \
             CASE WHEN image_base64 IS NULL THEN NULL ELSE CONCAT('/api/products/', id, '/image?v=', \
             DATE_FORMAT(updated_at, '%Y%m%d%H%i%s%f')) END AS image_url, \
             DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at, \
             DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at \
             FROM products WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn create(
        &self,
        input: ProductInput,
        image: Option<(String, String)>,
    ) -> Result<Product, sqlx::Error> {
        let (image_mime_type, image_base64) = image
            .as_ref()
            .map(|(mime_type, encoded)| (Some(mime_type.as_str()), Some(encoded.as_str())))
            .unwrap_or((None, None));
        let result = sqlx::query(
            "INSERT INTO products (name, category, description, specifications, price, stock, status, discount_eligible, image_mime_type, image_base64) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(input.name.trim())
        .bind(input.category.trim())
        .bind(input.description.trim())
        .bind(sqlx::types::Json(input.specifications))
        .bind(input.price)
        .bind(input.stock)
        .bind(input.status)
        .bind(input.discount_eligible)
        .bind(image_mime_type)
        .bind(image_base64)
        .execute(&self.pool)
        .await?;
        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub(crate) async fn update(
        &self,
        id: i64,
        input: ProductInput,
        image: Option<(String, String)>,
    ) -> Result<bool, sqlx::Error> {
        if self.find_by_id(id).await?.is_none() {
            return Ok(false);
        }
        let result = match image.as_ref() {
            Some((mime_type, encoded)) => sqlx::query(
                "UPDATE products SET name = ?, category = ?, description = ?, specifications = ?, price = ?, stock = ?, status = ?, discount_eligible = ?, \
                 image_mime_type = ?, image_base64 = ? WHERE id = ?",
            )
            .bind(input.name.trim())
            .bind(input.category.trim())
            .bind(input.description.trim())
            .bind(sqlx::types::Json(input.specifications))
            .bind(input.price)
            .bind(input.stock)
            .bind(input.status)
            .bind(input.discount_eligible)
            .bind(mime_type)
            .bind(encoded)
            .bind(id)
            .execute(&self.pool)
            .await?,
            None => sqlx::query(
                "UPDATE products SET name = ?, category = ?, description = ?, specifications = ?, price = ?, stock = ?, status = ?, discount_eligible = ? WHERE id = ?",
            )
            .bind(input.name.trim())
            .bind(input.category.trim())
            .bind(input.description.trim())
            .bind(sqlx::types::Json(input.specifications))
            .bind(input.price)
            .bind(input.stock)
            .bind(input.status)
            .bind(input.discount_eligible)
            .bind(id)
            .execute(&self.pool)
            .await?,
        };
        Ok(result.rows_affected() > 0 || self.find_by_id(id).await?.is_some())
    }

    pub(crate) async fn find_image(&self, id: i64) -> Result<Option<ProductImageRow>, sqlx::Error> {
        sqlx::query_as::<_, ProductImageRow>(
            "SELECT image_mime_type AS mime_type, image_base64 \
             FROM products WHERE id = ? AND image_mime_type IS NOT NULL AND image_base64 IS NOT NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
