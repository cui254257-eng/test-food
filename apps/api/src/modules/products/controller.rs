use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{app::AppState, common::ApiError};

use super::{
    dto::{ProductInput, ProductQuery},
    entity::Product,
};

pub(crate) async fn list(
    State(state): State<AppState>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>, ApiError> {
    Ok(Json(state.products.list(query).await?))
}

pub(crate) async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Product>, ApiError> {
    Ok(Json(state.products.get(id).await?))
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ProductInput>,
) -> Result<(StatusCode, Json<Product>), ApiError> {
    state.authenticate_admin(&headers)?;
    let product = state.products.create(input).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

pub(crate) async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<ProductInput>,
) -> Result<Json<Product>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.products.update(id, input).await?))
}

pub(crate) async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.authenticate_admin(&headers)?;
    state.products.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let image = state.products.image(id).await?;
    Ok((
        [
            (header::CONTENT_TYPE, image.mime_type),
            (header::CACHE_CONTROL, "public, max-age=3600".into()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        ],
        Body::from(image.bytes),
    )
        .into_response())
}
