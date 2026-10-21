use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
};

use crate::{app::AppState, common::ApiError};

use super::{
    dto::{UpdateBannerDto, UploadBannerDto},
    entity::Banner,
};

pub(crate) async fn list(State(state): State<AppState>) -> Result<Json<Vec<Banner>>, ApiError> {
    Ok(Json(state.banners.list().await?))
}

pub(crate) async fn upload(
    State(state): State<AppState>,
    Json(input): Json<UploadBannerDto>,
) -> Result<(axum::http::StatusCode, Json<Banner>), ApiError> {
    let banner = state.banners.upload(input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(banner)))
}

pub(crate) async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateBannerDto>,
) -> Result<Json<Banner>, ApiError> {
    Ok(Json(state.banners.update(id, input).await?))
}

pub(crate) async fn image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let image = state.banners.image(id).await?;
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

pub(crate) async fn background_image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let image = state.banners.background_image(id).await?;
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

pub(crate) async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, ApiError> {
    state.banners.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
