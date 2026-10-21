use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};

use crate::{app::AppState, common::ApiError};

use super::{
    dto::{AdminOrderQuery, OrderInput},
    entity::{AdminOrderPage, Order, OrderSummary},
};

pub(crate) async fn list_customer_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<OrderSummary>>, ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    let orders = state.orders.list_for_customer(customer_id).await?;
    Ok(Json(orders))
}

pub(crate) async fn list_admin_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminOrderQuery>,
) -> Result<Json<AdminOrderPage>, ApiError> {
    state.authenticate_admin(&headers)?;

    let page = state.orders.list_for_admin(query).await?;
    Ok(Json(page))
}

pub(crate) async fn mark_admin_order_paid(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
) -> Result<Json<super::entity::PaidOrderResult>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.orders.mark_paid(&order_number).await?))
}

pub(crate) async fn delete_admin_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.authenticate_admin(&headers)?;
    state.orders.delete_admin_order(&order_number).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<OrderInput>,
) -> Result<(StatusCode, Json<Order>), ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    let order = state.orders.create(customer_id, input).await?;
    Ok((StatusCode::CREATED, Json(order)))
}
