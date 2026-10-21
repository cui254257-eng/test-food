use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{dto::AddressInput, entity::Address, repository::AddressesRepository};

#[derive(Clone)]
pub(crate) struct AddressesService {
    repository: AddressesRepository,
}

impl AddressesService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: AddressesRepository::new(pool),
        }
    }

    pub(crate) async fn list(&self, customer_id: i64) -> Result<Vec<Address>, ApiError> {
        Ok(self.repository.list(customer_id).await?)
    }

    pub(crate) async fn create(
        &self,
        customer_id: i64,
        input: AddressInput,
    ) -> Result<Address, ApiError> {
        validate(&input)?;
        Ok(self.repository.create(customer_id, input).await?)
    }

    pub(crate) async fn update(
        &self,
        customer_id: i64,
        id: i64,
        input: AddressInput,
    ) -> Result<Address, ApiError> {
        validate(&input)?;
        self.repository
            .update(customer_id, id, input)
            .await?
            .ok_or_else(|| ApiError::not_found("收货地址不存在"))
    }

    pub(crate) async fn delete(&self, customer_id: i64, id: i64) -> Result<(), ApiError> {
        if !self.repository.delete(customer_id, id).await? {
            return Err(ApiError::not_found("收货地址不存在"));
        }
        Ok(())
    }
}

fn validate(input: &AddressInput) -> Result<(), ApiError> {
    let recipient_name = input.recipient_name.trim();
    if recipient_name.is_empty() || recipient_name.chars().count() > 40 {
        return Err(ApiError::bad_request(
            "收件人姓名不能为空且不能超过 40 个字",
        ));
    }
    let phone = input.phone.trim();
    if phone.len() != 11
        || !phone.starts_with('1')
        || !phone.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ApiError::bad_request("请输入有效的 11 位手机号码"));
    }
    for (name, value) in [
        ("省份", input.province.trim()),
        ("城市", input.city.trim()),
        ("区县", input.district.trim()),
    ] {
        if value.is_empty() || value.chars().count() > 50 {
            return Err(ApiError::bad_request(format!(
                "{name}不能为空且不能超过 50 个字"
            )));
        }
    }
    let detail = input.detail.trim();
    if detail.is_empty() || detail.chars().count() > 255 {
        return Err(ApiError::bad_request("详细地址不能为空且不能超过 255 个字"));
    }
    Ok(())
}
