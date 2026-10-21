use axum::{Json, extract::State};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
};

use crate::{app::AppState, common::ApiError};

use super::{
    dto::{AnnouncementInput, RestaurantSettingsInput},
    entity::{Announcement, RestaurantSettings},
};

pub(crate) async fn get(
    State(state): State<AppState>,
) -> Result<Json<RestaurantSettings>, ApiError> {
    Ok(Json(state.restaurant_settings.get().await?))
}

pub(crate) async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RestaurantSettingsInput>,
) -> Result<Json<RestaurantSettings>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.restaurant_settings.update(input).await?))
}

pub(crate) async fn announcements(
    State(state): State<AppState>,
) -> Result<Json<Vec<Announcement>>, ApiError> {
    Ok(Json(state.restaurant_settings.announcements().await?))
}

pub(crate) async fn admin_announcements(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Announcement>>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(state.restaurant_settings.all_announcements().await?))
}

pub(crate) async fn create_announcement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AnnouncementInput>,
) -> Result<(StatusCode, Json<Announcement>), ApiError> {
    state.authenticate_admin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(state.restaurant_settings.create_announcement(input).await?),
    ))
}

pub(crate) async fn update_announcement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<AnnouncementInput>,
) -> Result<Json<Announcement>, ApiError> {
    state.authenticate_admin(&headers)?;
    Ok(Json(
        state
            .restaurant_settings
            .update_announcement(id, input)
            .await?,
    ))
}

pub(crate) async fn delete_announcement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.authenticate_admin(&headers)?;
    state.restaurant_settings.delete_announcement(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
