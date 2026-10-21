use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::{app::AppState, common::ApiError};

use super::{dto::AddressInput, entity::Address};

pub(crate) async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Address>>, ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    Ok(Json(state.addresses.list(customer_id).await?))
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AddressInput>,
) -> Result<(StatusCode, Json<Address>), ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    let address = state.addresses.create(customer_id, input).await?;
    Ok((StatusCode::CREATED, Json(address)))
}

pub(crate) async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<AddressInput>,
) -> Result<Json<Address>, ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    Ok(Json(state.addresses.update(customer_id, id, input).await?))
}

pub(crate) async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let customer_id = state.auth.authenticate(&headers).await?;
    state.addresses.delete(customer_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
