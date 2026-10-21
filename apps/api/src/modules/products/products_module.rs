use axum::{Router, extract::DefaultBodyLimit, routing::get};

use crate::app::AppState;

use super::controller;

/// Registers the products feature endpoints, like a NestJS feature module.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/products/{id}/image", get(controller::image))
        .route(
            "/api/products",
            get(controller::list).post(controller::create),
        )
        .route(
            "/api/products/{id}",
            get(controller::get)
                .put(controller::update)
                .delete(controller::delete),
        )
        .layer(DefaultBodyLimit::max(7 * 1024 * 1024))
}
