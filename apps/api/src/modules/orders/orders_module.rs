use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/orders", post(controller::create))
}
