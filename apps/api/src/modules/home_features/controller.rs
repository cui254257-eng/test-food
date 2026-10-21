use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{app::AppState, common::ApiError};

use super::{dto::HomeFeatureInput, entity::HomeFeature};

pub(crate) async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<HomeFeature>>, ApiError> {
    Ok(Json(state.home_features.list().await?))
}

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(input): Json<HomeFeatureInput>,
) -> Result<(StatusCode, Json<HomeFeature>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(state.home_features.create(input).await?),
    ))
}

pub(crate) async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<HomeFeatureInput>,
) -> Result<Json<HomeFeature>, ApiError> {
    Ok(Json(state.home_features.update(id, input).await?))
}

pub(crate) async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.home_features.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
