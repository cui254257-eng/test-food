use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct MembershipTier {
    pub(crate) level: i64,
    pub(crate) required_points: i64,
    pub(crate) discount_rate: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct MembershipConfig {
    pub(crate) spend_step_cents: i64,
    pub(crate) points_per_step: i64,
    pub(crate) tiers: Vec<MembershipTier>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct AdminCustomerQuery {
    pub(crate) q: Option<String>,
    pub(crate) page: Option<u32>,
    pub(crate) page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminCustomerPage {
    pub(crate) customers: Vec<AdminCustomer>,
    pub(crate) total: i64,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub(crate) struct AdminCustomer {
    pub(crate) id: i64,
    pub(crate) nickname: String,
    pub(crate) points: i64,
    pub(crate) member_level: i64,
    pub(crate) discount_rate: f64,
    pub(crate) order_count: i64,
    pub(crate) paid_total_cents: i64,
    pub(crate) created_at: String,
}
