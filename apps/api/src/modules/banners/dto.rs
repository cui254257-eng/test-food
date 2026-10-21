use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct UploadBannerDto {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) sort_order: i32,
    pub(crate) image_data: String,
    #[serde(default)]
    pub(crate) background_image_data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateBannerDto {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) sort_order: i32,
    pub(crate) image_data: Option<String>,
    #[serde(default)]
    pub(crate) background_image_data: Option<String>,
}
