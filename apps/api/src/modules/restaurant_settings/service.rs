use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{
    dto::{AnnouncementInput, RestaurantSettingsInput},
    entity::{Announcement, RestaurantSettings},
    repository::RestaurantSettingsRepository,
};

#[derive(Clone)]
pub(crate) struct RestaurantSettingsService {
    repository: RestaurantSettingsRepository,
}

impl RestaurantSettingsService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: RestaurantSettingsRepository::new(pool),
        }
    }

    pub(crate) async fn get(&self) -> Result<RestaurantSettings, ApiError> {
        Ok(self.repository.get().await?)
    }

    pub(crate) async fn update(
        &self,
        input: RestaurantSettingsInput,
    ) -> Result<RestaurantSettings, ApiError> {
        validate(&input)?;
        Ok(self.repository.update(input).await?)
    }

    pub(crate) async fn announcements(&self) -> Result<Vec<Announcement>, ApiError> {
        Ok(self.repository.get_announcements().await?)
    }

    pub(crate) async fn all_announcements(&self) -> Result<Vec<Announcement>, ApiError> {
        Ok(self.repository.get_all_announcements().await?)
    }

    pub(crate) async fn create_announcement(
        &self,
        input: AnnouncementInput,
    ) -> Result<Announcement, ApiError> {
        validate_announcement(&input)?;
        Ok(self.repository.create_announcement(input).await?)
    }

    pub(crate) async fn update_announcement(
        &self,
        id: i64,
        input: AnnouncementInput,
    ) -> Result<Announcement, ApiError> {
        validate_announcement(&input)?;
        self.repository
            .update_announcement(id, input)
            .await?
            .ok_or_else(|| ApiError::not_found("公告不存在"))
    }

    pub(crate) async fn delete_announcement(&self, id: i64) -> Result<(), ApiError> {
        if !self.repository.delete_announcement(id).await? {
            return Err(ApiError::not_found("公告不存在"));
        }
        Ok(())
    }
}

fn validate(input: &RestaurantSettingsInput) -> Result<(), ApiError> {
    if input.restaurant_name.trim().is_empty() || input.restaurant_name.chars().count() > 80 {
        return Err(ApiError::bad_request("门店名称不能为空且不能超过 80 个字"));
    }
    if input.business_hours.trim().is_empty()
        || input.business_hours.chars().count() > 255
        || input.contact_phone.chars().count() > 20
        || input.store_address.chars().count() > 255
    {
        return Err(ApiError::bad_request("营业信息或联系信息长度不正确"));
    }
    if !input.delivery_radius_km.is_finite()
        || input.delivery_radius_km < 0.1
        || input.delivery_radius_km > 100.0
        || !input.delivery_minimum_amount.is_finite()
        || input.delivery_minimum_amount < 0.0
        || !input.delivery_fee.is_finite()
        || input.delivery_fee < 0.0
    {
        return Err(ApiError::bad_request(
            "配送范围、起送金额和配送费必须是有效的非负数",
        ));
    }
    if input.delivery_enabled && input.store_address.trim().is_empty() {
        return Err(ApiError::bad_request("启用外卖配送前请填写门店地址"));
    }
    if !input.dine_in_enabled && !input.takeaway_enabled && !input.delivery_enabled {
        return Err(ApiError::bad_request(
            "堂食、外带或外卖至少保留一种服务方式",
        ));
    }
    Ok(())
}

fn validate_announcement(input: &AnnouncementInput) -> Result<(), ApiError> {
    let content = input.content.trim();
    if content.is_empty() || content.chars().count() > 255 {
        return Err(ApiError::bad_request("公告内容不能为空且不能超过 255 个字"));
    }
    if input.sort_order < 0 {
        return Err(ApiError::bad_request("公告排序值不能小于 0"));
    }
    Ok(())
}
