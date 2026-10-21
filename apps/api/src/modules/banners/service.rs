use base64::{Engine as _, engine::general_purpose::STANDARD};
use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{
    dto::{UpdateBannerDto, UploadBannerDto},
    entity::{Banner, BannerImage},
    repository::BannersRepository,
};

const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct BannersService {
    repository: BannersRepository,
}

impl BannersService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: BannersRepository::new(pool),
        }
    }

    pub(crate) async fn list(&self) -> Result<Vec<Banner>, ApiError> {
        Ok(self.repository.list().await?)
    }

    pub(crate) async fn upload(&self, input: UploadBannerDto) -> Result<Banner, ApiError> {
        let title = input.title.trim();
        validate_title(title)?;
        validate_sort_order(input.sort_order)?;

        let (mime_type, image_base64) = validate_image_data(&input.image_data)?;
        let background_image = input
            .background_image_data
            .as_deref()
            .map(validate_image_data)
            .transpose()?;
        Ok(self
            .repository
            .create(
                title,
                input.sort_order,
                mime_type,
                image_base64,
                background_image,
            )
            .await?)
    }

    pub(crate) async fn update(&self, id: i64, input: UpdateBannerDto) -> Result<Banner, ApiError> {
        let title = input.title.trim();
        validate_title(title)?;
        validate_sort_order(input.sort_order)?;

        let image = input
            .image_data
            .as_deref()
            .map(validate_image_data)
            .transpose()?;
        let background_image = input
            .background_image_data
            .as_deref()
            .map(validate_image_data)
            .transpose()?;
        self.repository
            .update(id, title, input.sort_order, image, background_image)
            .await?
            .ok_or_else(|| ApiError::not_found("轮播图不存在"))
    }

    pub(crate) async fn background_image(&self, id: i64) -> Result<BannerImage, ApiError> {
        let row = self
            .repository
            .find_background_image(id)
            .await?
            .ok_or_else(|| ApiError::not_found("轮播图背景图不存在"))?;
        let bytes = STANDARD
            .decode(row.image_base64)
            .map_err(|_| ApiError::bad_request("轮播图背景图数据无效"))?;
        Ok(BannerImage {
            mime_type: row.mime_type,
            bytes,
        })
    }

    pub(crate) async fn image(&self, id: i64) -> Result<BannerImage, ApiError> {
        let row = self
            .repository
            .find_image(id)
            .await?
            .ok_or_else(|| ApiError::not_found("轮播图不存在"))?;
        let bytes = STANDARD
            .decode(row.image_base64)
            .map_err(|_| ApiError::bad_request("轮播图数据无效"))?;
        Ok(BannerImage {
            mime_type: row.mime_type,
            bytes,
        })
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<(), ApiError> {
        if !self.repository.delete(id).await? {
            return Err(ApiError::not_found("轮播图不存在"));
        }
        Ok(())
    }
}

fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.chars().count() > 80 {
        return Err(ApiError::bad_request("轮播图标题不能超过 80 个字"));
    }
    Ok(())
}

fn validate_sort_order(sort_order: i32) -> Result<(), ApiError> {
    if sort_order < 0 {
        return Err(ApiError::bad_request("排序值不能小于 0"));
    }
    Ok(())
}

fn validate_image_data(data_url: &str) -> Result<(&'static str, &str), ApiError> {
    let (header, encoded) = data_url
        .split_once(',')
        .ok_or_else(|| ApiError::bad_request("请上传 PNG、JPG、WEBP 或 GIF 图片"))?;

    let mime_type = match header {
        "data:image/png;base64" => "image/png",
        "data:image/jpeg;base64" => "image/jpeg",
        "data:image/webp;base64" => "image/webp",
        "data:image/gif;base64" => "image/gif",
        _ => return Err(ApiError::bad_request("图片格式不支持")),
    };

    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| ApiError::bad_request("图片数据无效"))?;
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err(ApiError::bad_request("图片大小需小于 5 MB"));
    }

    let matches_type = match mime_type {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(b"\xff\xd8\xff"),
        "image/webp" => bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        _ => false,
    };
    if !matches_type {
        return Err(ApiError::bad_request("图片格式与文件内容不一致"));
    }

    Ok((mime_type, encoded))
}
