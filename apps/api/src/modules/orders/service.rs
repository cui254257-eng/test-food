use sqlx::MySqlPool;

use crate::common::ApiError;

use super::{
    dto::{AdminOrderQuery, OrderInput},
    entity::{AdminOrderPage, Order, OrderSummary, PaidOrderResult},
    repository::OrdersRepository,
};

#[derive(Clone)]
pub(crate) struct OrdersService {
    repository: OrdersRepository,
}

impl OrdersService {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            repository: OrdersRepository::new(pool),
        }
    }

    pub(crate) async fn create(
        &self,
        customer_id: i64,
        mut input: OrderInput,
    ) -> Result<Order, ApiError> {
        validate(&mut input)?;
        self.repository.create(customer_id, input).await
    }

    pub(crate) async fn list_for_customer(
        &self,
        customer_id: i64,
    ) -> Result<Vec<OrderSummary>, ApiError> {
        self.repository.list_for_customer(customer_id).await
    }

    pub(crate) async fn list_for_admin(
        &self,
        query: AdminOrderQuery,
    ) -> Result<AdminOrderPage, ApiError> {
        let query = validate_admin_query(query)?;
        self.repository.list_for_admin(query).await
    }

    pub(crate) async fn mark_paid(&self, order_number: &str) -> Result<PaidOrderResult, ApiError> {
        self.repository.mark_paid(order_number).await
    }

    pub(crate) async fn delete_admin_order(&self, order_number: &str) -> Result<(), ApiError> {
        self.repository.delete_admin_order(order_number).await
    }

    pub(crate) async fn expire_pending_orders(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
        super::repository::expire_pending_orders(pool).await
    }
}

fn validate_admin_query(mut query: AdminOrderQuery) -> Result<AdminOrderQuery, ApiError> {
    query.status = query
        .status
        .take()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if query
        .status
        .as_ref()
        .is_some_and(|value| value.len() > 24 || value.chars().any(char::is_control))
    {
        return Err(ApiError::bad_request("订单状态筛选条件无效"));
    }

    query.fulfillment_mode = query
        .fulfillment_mode
        .take()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if query
        .fulfillment_mode
        .as_ref()
        .is_some_and(|value| !matches!(value.as_str(), "dine_in" | "takeaway" | "delivery"))
    {
        return Err(ApiError::bad_request("用餐方式筛选条件无效"));
    }

    if query.customer_id.is_some_and(|customer_id| customer_id < 1) {
        return Err(ApiError::bad_request("用户筛选条件无效"));
    }

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

    query.page = Some(query.page.unwrap_or(1).max(1));
    query.page_size = Some(query.page_size.unwrap_or(20).clamp(10, 100));
    Ok(query)
}

fn validate(input: &mut OrderInput) -> Result<(), ApiError> {
    input.request_key = input.request_key.trim().to_owned();
    if !(8..=64).contains(&input.request_key.len())
        || !input
            .request_key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ApiError::bad_request("下单请求标识无效，请重新提交"));
    }
    if !matches!(
        input.fulfillment_mode.as_str(),
        "dine_in" | "takeaway" | "delivery"
    ) {
        return Err(ApiError::bad_request("请选择有效的用餐方式"));
    }
    if input.items.is_empty() || input.items.len() > 30 {
        return Err(ApiError::bad_request("点餐篮为空或菜品种类过多"));
    }

    if input.fulfillment_mode == "dine_in" {
        let table_number = input.table_number.as_deref().unwrap_or_default().trim();
        if table_number.is_empty() || table_number.chars().count() > 20 {
            return Err(ApiError::bad_request("请填写桌号（最多 20 个字）"));
        }
        input.table_number = Some(table_number.to_owned());
        input.address_id = None;
    } else if input.fulfillment_mode == "delivery" {
        if !input.address_id.is_some_and(|id| id > 0) {
            return Err(ApiError::bad_request("请选择外卖收货地址"));
        }
        input.table_number = None;
    } else {
        input.table_number = None;
        input.address_id = None;
    }

    for item in &mut input.items {
        if item.product_id < 1 || !(1..=99).contains(&item.quantity) || item.selections.len() > 10 {
            return Err(ApiError::bad_request("菜品、数量或规格信息无效"));
        }
        for selection in &mut item.selections {
            selection.group_name = selection.group_name.trim().to_owned();
            selection.option_name = selection.option_name.trim().to_owned();
            if selection.group_name.is_empty() || selection.option_name.is_empty() {
                return Err(ApiError::bad_request("菜品规格信息无效，请重新选择"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate, validate_admin_query};
    use crate::modules::orders::dto::{AdminOrderQuery, OrderInput, OrderItemInput};

    fn sample() -> OrderInput {
        OrderInput {
            request_key: "order-123456".into(),
            fulfillment_mode: "takeaway".into(),
            table_number: None,
            address_id: None,
            items: vec![OrderItemInput {
                product_id: 1,
                quantity: 1,
                selections: vec![],
            }],
        }
    }

    #[test]
    fn accepts_takeaway_and_trims_idempotency_key() {
        let mut input = sample();
        input.request_key = " order-123456 ".into();
        assert!(validate(&mut input).is_ok());
        assert_eq!(input.request_key, "order-123456");
    }

    #[test]
    fn requires_a_table_for_dine_in() {
        let mut input = sample();
        input.fulfillment_mode = "dine_in".into();
        assert!(validate(&mut input).is_err());
        input.table_number = Some("A08".into());
        assert!(validate(&mut input).is_ok());
    }

    #[test]
    fn requires_address_for_delivery_and_bounded_quantities() {
        let mut input = sample();
        input.fulfillment_mode = "delivery".into();
        assert!(validate(&mut input).is_err());
        input.address_id = Some(2);
        input.items[0].quantity = 100;
        assert!(validate(&mut input).is_err());
    }

    #[test]
    fn normalizes_admin_filters_and_bounds_page_size() {
        let query = AdminOrderQuery {
            q: Some("  user 42 ".into()),
            page: Some(0),
            page_size: Some(1000),
            ..AdminOrderQuery::default()
        };
        let query = validate_admin_query(query).unwrap();
        assert_eq!(query.q.as_deref(), Some("user 42"));
        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(100));
    }

    #[test]
    fn rejects_invalid_admin_fulfillment_filter() {
        let query = AdminOrderQuery {
            fulfillment_mode: Some("unknown".into()),
            ..AdminOrderQuery::default()
        };
        assert!(validate_admin_query(query).is_err());
    }
}
