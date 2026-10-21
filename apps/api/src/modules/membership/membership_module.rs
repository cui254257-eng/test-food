use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/membership/config", get(controller::public_config))
        .route(
            "/api/admin/membership/config",
            get(controller::admin_config).put(controller::update_config),
        )
        .route("/api/admin/customers", get(controller::list_customers))
}
