use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};

use crate::{app::AppState, common::ApiError};

use super::dto::{AdminCustomerPage, AdminCustomerQuery, MembershipConfig};

pub(crate) async fn public_config(
    State(state): State<AppState>,
) -> Result<Json<MembershipConfig>, ApiError> {
    Ok(Json(state.membership.config().await?))
}

pub(crate) async fn admin_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MembershipConfig>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.membership.config().await?))
}

pub(crate) async fn update_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(config): Json<MembershipConfig>,
) -> Result<Json<MembershipConfig>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.membership.update_config(config).await?))
}

pub(crate) async fn list_customers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminCustomerQuery>,
) -> Result<Json<AdminCustomerPage>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.membership.list_customers(query).await?))
}
