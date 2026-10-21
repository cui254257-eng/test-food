use serde::Serialize;
use sqlx::{FromRow, types::Json};

use super::dto::ProductSpecificationGroup;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Product {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) category: String,
    pub(crate) description: String,
    pub(crate) specifications: Json<Vec<ProductSpecificationGroup>>,
    pub(crate) price: f64,
    pub(crate) stock: i64,
    pub(crate) status: String,
    pub(crate) image_url: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, FromRow)]
pub(crate) struct ProductImageRow {
    pub(crate) mime_type: String,
    pub(crate) image_base64: String,
}

pub(crate) struct ProductImage {
    pub(crate) mime_type: String,
    pub(crate) bytes: Vec<u8>,
}
