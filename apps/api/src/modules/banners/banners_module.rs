use axum::{Router, extract::DefaultBodyLimit, routing::get};

use crate::app::AppState;

use super::controller;

// Two independently validated 5 MB images are Base64-encoded in one JSON request.
const MAX_UPLOAD_BODY: usize = 14 * 1024 * 1024;

/// Registers banner listing, image upload, image delivery, and deletion routes.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/banners",
            get(controller::list).post(controller::upload),
        )
        .route("/api/banners/{id}/image", get(controller::image))
        .route(
            "/api/banners/{id}/background-image",
            get(controller::background_image),
        )
        .route(
            "/api/banners/{id}",
            axum::routing::put(controller::update).delete(controller::delete),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BODY))
}
