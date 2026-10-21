use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/home-features",
            get(controller::list).post(controller::create),
        )
        .route(
            "/api/home-features/{id}",
            axum::routing::put(controller::update).delete(controller::delete),
        )
}
