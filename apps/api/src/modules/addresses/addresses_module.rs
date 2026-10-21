use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/addresses",
            get(controller::list).post(controller::create),
        )
        .route(
            "/api/addresses/{id}",
            axum::routing::put(controller::update).delete(controller::delete),
        )
}
