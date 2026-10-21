use base64::{Engine as _, engine::general_purpose::STANDARD};
use sqlx::MySqlPool;
use std::collections::HashSet;

use crate::common::ApiError;

use super::{
    dto::{ProductInput, ProductQuery},
    entity::{Product, ProductImage},
    repository::ProductsRepository,
};

#[derive(Clone)]
pub(crate) struct ProductsService {
    repository: ProductsRepository,
}

impl ProductsService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: ProductsRepository::new(pool),
        }
    }

    pub(crate) async fn list(&self, query: ProductQuery) -> Result<Vec<Product>, ApiError> {
        Ok(self.repository.list(query).await?)
    }

    pub(crate) async fn get(&self, id: i64) -> Result<Product, ApiError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApiError::not_found("菜品不存在"))
    }

    pub(crate) async fn create(&self, input: ProductInput) -> Result<Product, ApiError> {
        validate(&input)?;
        let image = input
            .image_data
            .as_deref()
            .map(validate_image_data)
            .transpose()?;
        Ok(self.repository.create(input, image).await?)
    }

    pub(crate) async fn update(&self, id: i64, input: ProductInput) -> Result<Product, ApiError> {
        validate(&input)?;
        let image = input
            .image_data
            .as_deref()
            .map(validate_image_data)
            .transpose()?;
        if !self.repository.update(id, input, image).await? {
            return Err(ApiError::not_found("菜品不存在"));
        }
        self.get(id).await
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<(), ApiError> {
        if !self.repository.delete(id).await? {
            return Err(ApiError::not_found("菜品不存在"));
        }
        Ok(())
    }

    pub(crate) async fn image(&self, id: i64) -> Result<ProductImage, ApiError> {
        let row = self
            .repository
            .find_image(id)
            .await?
            .ok_or_else(|| ApiError::not_found("菜品图片不存在"))?;
        let bytes = STANDARD
            .decode(row.image_base64)
            .map_err(|_| ApiError::bad_request("菜品图片数据无效"))?;
        Ok(ProductImage {
            mime_type: row.mime_type,
            bytes,
        })
    }
}

fn validate(input: &ProductInput) -> Result<(), ApiError> {
    if input.name.trim().is_empty() {
        return Err(ApiError::bad_request("菜品名称不能为空"));
    }
    if input.name.chars().count() > 80 {
        return Err(ApiError::bad_request("菜品名称不能超过 80 个字"));
    }
    if input.category.trim().is_empty() || input.category.chars().count() > 40 {
        return Err(ApiError::bad_request("菜品分类不能为空且不能超过 40 个字"));
    }
    if input.description.chars().count() > 500 {
        return Err(ApiError::bad_request("菜品介绍不能超过 500 个字"));
    }
    if !input.price.is_finite() || input.price < 0.0 {
        return Err(ApiError::bad_request("菜品售价必须是大于等于 0 的有效数字"));
    }
    if input.stock < 0 {
        return Err(ApiError::bad_request("菜品可售份数不能小于 0"));
    }
    if input.specifications.len() > 10 {
        return Err(ApiError::bad_request("每道菜最多配置 10 组规格"));
    }
    let mut group_names = HashSet::new();
    for group in &input.specifications {
        if group.name.trim().is_empty() || group.name.chars().count() > 30 {
            return Err(ApiError::bad_request("规格名称不能为空且不能超过 30 个字"));
        }
        if !group_names.insert(group.name.trim()) {
            return Err(ApiError::bad_request("同一道菜的规格组名称不能重复"));
        }
        if group.options.is_empty() || group.options.len() > 20 {
            return Err(ApiError::bad_request("每组规格需配置 1 到 20 个选项"));
        }
        let mut option_names = HashSet::new();
        for option in &group.options {
            if option.name.trim().is_empty()
                || option.name.chars().count() > 50
                || !option.price_delta.is_finite()
                || option.price_delta < 0.0
            {
                return Err(ApiError::bad_request("规格选项名称或加价设置无效"));
            }
            if !option_names.insert(option.name.trim()) {
                return Err(ApiError::bad_request("同一规格组选项名称不能重复"));
            }
        }
    }
    if !matches!(input.status.as_str(), "active" | "inactive") {
        return Err(ApiError::bad_request("菜品状态必须为 active 或 inactive"));
    }
    Ok(())
}

fn validate_image_data(data_url: &str) -> Result<(String, String), ApiError> {
    const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
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
    Ok((mime_type.to_owned(), encoded.to_owned()))
}
