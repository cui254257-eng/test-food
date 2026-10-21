use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub(crate) struct AdminOrderQuery {
    pub(crate) status: Option<String>,
    pub(crate) fulfillment_mode: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) customer_id: Option<i64>,
    pub(crate) page: Option<u32>,
    pub(crate) page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrderInput {
    pub(crate) request_key: String,
    pub(crate) fulfillment_mode: String,
    pub(crate) table_number: Option<String>,
    pub(crate) address_id: Option<i64>,
    pub(crate) items: Vec<OrderItemInput>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrderItemInput {
    pub(crate) product_id: i64,
    pub(crate) quantity: i64,
    #[serde(default)]
    pub(crate) selections: Vec<SpecificationSelection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SpecificationSelection {
    pub(crate) group_name: String,
    pub(crate) option_name: String,
}
