use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{app::AppState, common::ApiError};

use super::{dto::OrderInput, entity::Order};

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<OrderInput>,
) -> Result<(StatusCode, Json<Order>), ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    let order = state.orders.create(customer_id, input).await?;
    Ok((StatusCode::CREATED, Json(order)))
}
