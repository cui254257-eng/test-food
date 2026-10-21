use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{dto::HomeFeatureInput, entity::HomeFeature, repository::HomeFeaturesRepository};

#[derive(Clone)]
pub(crate) struct HomeFeaturesService {
    repository: HomeFeaturesRepository,
}

impl HomeFeaturesService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: HomeFeaturesRepository::new(pool),
        }
    }

    pub(crate) async fn list(&self) -> Result<Vec<HomeFeature>, ApiError> {
        Ok(self.repository.list().await?)
    }

    pub(crate) async fn create(&self, input: HomeFeatureInput) -> Result<HomeFeature, ApiError> {
        validate(&input)?;
        Ok(self.repository.create(input).await?)
    }

    pub(crate) async fn update(
        &self,
        id: i64,
        input: HomeFeatureInput,
    ) -> Result<HomeFeature, ApiError> {
        validate(&input)?;
        self.repository
            .update(id, input)
            .await?
            .ok_or_else(|| ApiError::not_found("点餐分类不存在"))
    }

    pub(crate) async fn delete(&self, id: i64) -> Result<(), ApiError> {
        if !self.repository.delete(id).await? {
            return Err(ApiError::not_found("点餐分类不存在"));
        }
        Ok(())
    }
}

fn validate(input: &HomeFeatureInput) -> Result<(), ApiError> {
    let label = input.label.trim();
    if label.is_empty() || label.chars().count() > 40 {
        return Err(ApiError::bad_request("模块名称不能为空且不能超过 40 个字"));
    }
    if !matches!(
        input.icon.as_str(),
        "all.svg"
            | "cart.svg"
            | "coffee.svg"
            | "home.svg"
            | "hot.svg"
            | "new.svg"
            | "price.svg"
            | "snacks.svg"
    ) {
        return Err(ApiError::bad_request("请选择有效的首页分类图标"));
    }
    let target = input.target_path.trim();
    if !target.starts_with("/pages/")
        || target.len() > 255
        || target.chars().any(char::is_whitespace)
    {
        return Err(ApiError::bad_request(
            "跳转地址须为有效的小程序页面路径，例如 /pages/cart/index",
        ));
    }
    if input.sort_order < 0 {
        return Err(ApiError::bad_request("排序值不能小于 0"));
    }
    Ok(())
}
