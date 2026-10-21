use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Banner {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) sort_order: i32,
    pub(crate) image_url: String,
    pub(crate) background_image_url: Option<String>,
}

#[derive(Debug, FromRow)]
pub(crate) struct BannerImageRow {
    pub(crate) mime_type: String,
    pub(crate) image_base64: String,
}

pub(crate) struct BannerImage {
    pub(crate) mime_type: String,
    pub(crate) bytes: Vec<u8>,
}
