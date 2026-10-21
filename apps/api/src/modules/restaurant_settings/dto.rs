use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct RestaurantSettingsInput {
    pub(crate) restaurant_name: String,
    pub(crate) business_hours: String,
    #[serde(default)]
    pub(crate) contact_phone: String,
    #[serde(default)]
    pub(crate) store_address: String,
    pub(crate) is_open: bool,
    pub(crate) dine_in_enabled: bool,
    pub(crate) takeaway_enabled: bool,
    pub(crate) delivery_enabled: bool,
    pub(crate) delivery_radius_km: f64,
    pub(crate) delivery_minimum_amount: f64,
    pub(crate) delivery_fee: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnnouncementInput {
    pub(crate) content: String,
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) sort_order: i32,
}
