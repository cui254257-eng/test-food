use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct RestaurantSettings {
    pub(crate) restaurant_name: String,
    pub(crate) business_hours: String,
    pub(crate) contact_phone: String,
    pub(crate) store_address: String,
    pub(crate) is_open: bool,
    pub(crate) dine_in_enabled: bool,
    pub(crate) takeaway_enabled: bool,
    pub(crate) delivery_enabled: bool,
    pub(crate) delivery_radius_km: f64,
    pub(crate) delivery_minimum_amount: f64,
    pub(crate) delivery_fee: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Announcement {
    pub(crate) id: i64,
    pub(crate) content: String,
    pub(crate) is_active: bool,
    pub(crate) sort_order: i32,
    pub(crate) updated_at: String,
}
