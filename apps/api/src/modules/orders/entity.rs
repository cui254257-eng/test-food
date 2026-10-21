use serde::Serialize;
use sqlx::{FromRow, types::Json};

use super::dto::SpecificationSelection;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Order {
    pub(crate) order_number: String,
    pub(crate) fulfillment_mode: String,
    pub(crate) status: String,
    pub(crate) payment_status: String,
    pub(crate) subtotal_cents: i64,
    pub(crate) discount_cents: i64,
    pub(crate) delivery_fee_cents: i64,
    pub(crate) total_cents: i64,
    pub(crate) points_earned: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct OrderSummary {
    pub(crate) order_number: String,
    pub(crate) customer_id: i64,
    pub(crate) customer_name: String,
    pub(crate) fulfillment_mode: String,
    pub(crate) table_number: Option<String>,
    pub(crate) delivery_recipient_name: Option<String>,
    pub(crate) delivery_phone: Option<String>,
    pub(crate) delivery_address: Option<String>,
    pub(crate) status: String,
    pub(crate) payment_status: String,
    pub(crate) subtotal_cents: i64,
    pub(crate) discount_cents: i64,
    pub(crate) delivery_fee_cents: i64,
    pub(crate) total_cents: i64,
    pub(crate) points_earned: i64,
    pub(crate) expires_at: String,
    pub(crate) created_at: String,
    pub(crate) items: Vec<OrderItemSummary>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OrderItemSummary {
    pub(crate) product_name: String,
    pub(crate) quantity: i64,
    pub(crate) unit_price_cents: i64,
    pub(crate) line_total_cents: i64,
    pub(crate) discount_cents: i64,
    pub(crate) discount_eligible: bool,
    pub(crate) selected_options: Vec<SpecificationSelection>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminOrderPage {
    pub(crate) orders: Vec<OrderSummary>,
    pub(crate) total: i64,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
}

#[derive(Debug, FromRow)]
pub(crate) struct OrderRecord {
    pub(crate) id: i64,
    pub(crate) order_number: String,
    pub(crate) customer_id: i64,
    pub(crate) customer_name: String,
    pub(crate) fulfillment_mode: String,
    pub(crate) table_number: Option<String>,
    pub(crate) delivery_recipient_name: Option<String>,
    pub(crate) delivery_phone: Option<String>,
    pub(crate) delivery_address: Option<String>,
    pub(crate) status: String,
    pub(crate) payment_status: String,
    pub(crate) subtotal_cents: i64,
    pub(crate) discount_cents: i64,
    pub(crate) delivery_fee_cents: i64,
    pub(crate) total_cents: i64,
    pub(crate) points_earned: i64,
    pub(crate) expires_at: String,
    pub(crate) created_at: String,
}

#[derive(Debug, FromRow)]
pub(crate) struct OrderItemRecord {
    pub(crate) order_id: i64,
    pub(crate) product_name: String,
    pub(crate) quantity: i64,
    pub(crate) unit_price_cents: i64,
    pub(crate) line_total_cents: i64,
    pub(crate) discount_cents: i64,
    pub(crate) discount_eligible: bool,
    pub(crate) selected_options: Json<Vec<SpecificationSelection>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PaidOrderResult {
    pub(crate) order_number: String,
    pub(crate) payment_status: String,
    pub(crate) status: String,
    pub(crate) points_earned: i64,
}
