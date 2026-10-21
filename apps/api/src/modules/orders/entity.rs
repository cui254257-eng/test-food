use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Order {
    pub(crate) order_number: String,
    pub(crate) fulfillment_mode: String,
    pub(crate) status: String,
    pub(crate) payment_status: String,
    pub(crate) subtotal_cents: i64,
    pub(crate) delivery_fee_cents: i64,
    pub(crate) total_cents: i64,
}
