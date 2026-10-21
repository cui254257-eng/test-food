use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{
    dto::{AdminCustomerPage, AdminCustomerQuery, MembershipConfig},
    repository::MembershipRepository,
};

#[derive(Clone)]
pub(crate) struct MembershipService {
    repository: MembershipRepository,
}

impl MembershipService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: MembershipRepository::new(pool),
        }
    }

    pub(crate) async fn config(&self) -> Result<MembershipConfig, ApiError> {
        Ok(self.repository.config().await?)
    }

    pub(crate) async fn update_config(
        &self,
        config: MembershipConfig,
    ) -> Result<MembershipConfig, ApiError> {
        validate_config(&config)?;
        self.repository.update_config(config).await?;
        self.config().await
    }

    pub(crate) async fn list_customers(
        &self,
        mut query: AdminCustomerQuery,
    ) -> Result<AdminCustomerPage, ApiError> {
        query.q = query
            .q
            .take()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if query
            .q
            .as_ref()
            .is_some_and(|value| value.chars().count() > 80 || value.chars().any(char::is_control))
        {
            return Err(ApiError::bad_request("搜索内容不能超过 80 个字符"));
        }
        Ok(self.repository.list_customers(query).await?)
    }
}

fn validate_config(config: &MembershipConfig) -> Result<(), ApiError> {
    if !(1..=10_000_000).contains(&config.spend_step_cents)
        || !(1..=1_000_000).contains(&config.points_per_step)
    {
        return Err(ApiError::bad_request("积分规则的消费金额或积分数量无效"));
    }
    if config.tiers.len() != 5 {
        return Err(ApiError::bad_request("会员等级必须完整配置 V1 到 V5"));
    }
    let mut previous_points = -1_i64;
    let mut previous_discount = 1.0_f64;
    for (index, tier) in config.tiers.iter().enumerate() {
        let expected_level = (index + 1) as i64;
        if tier.level != expected_level
            || tier.required_points < 0
            || tier.required_points <= previous_points
            || !tier.discount_rate.is_finite()
            || !(0.5..=1.0).contains(&tier.discount_rate)
            || tier.discount_rate > previous_discount
        {
            return Err(ApiError::bad_request(
                "等级积分门槛需递增，折扣需在 5 折至原价之间且等级越高折扣不变差",
            ));
        }
        if index == 0 && (tier.required_points != 0 || tier.discount_rate != 1.0) {
            return Err(ApiError::bad_request("V1 必须从 0 积分开始且不打折"));
        }
        previous_points = tier.required_points;
        previous_discount = tier.discount_rate;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_config;
    use crate::modules::membership::dto::{MembershipConfig, MembershipTier};

    fn sample() -> MembershipConfig {
        MembershipConfig {
            spend_step_cents: 100,
            points_per_step: 1,
            tiers: vec![
                MembershipTier {
                    level: 1,
                    required_points: 0,
                    discount_rate: 1.0,
                },
                MembershipTier {
                    level: 2,
                    required_points: 100,
                    discount_rate: 0.98,
                },
                MembershipTier {
                    level: 3,
                    required_points: 300,
                    discount_rate: 0.95,
                },
                MembershipTier {
                    level: 4,
                    required_points: 1000,
                    discount_rate: 0.93,
                },
                MembershipTier {
                    level: 5,
                    required_points: 3000,
                    discount_rate: 0.90,
                },
            ],
        }
    }

    #[test]
    fn accepts_default_tiers_and_rejects_bad_ordering() {
        let mut config = sample();
        assert!(validate_config(&config).is_ok());
        config.tiers[3].required_points = 200;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn v1_is_always_full_price() {
        let mut config = sample();
        config.tiers[0].discount_rate = 0.98;
        assert!(validate_config(&config).is_err());
    }
}
