use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/restaurant/settings",
            get(controller::get).put(controller::update),
        )
        .route("/api/announcements", get(controller::announcements))
        .route(
            "/api/admin/announcements",
            get(controller::admin_announcements).post(controller::create_announcement),
        )
        .route(
            "/api/admin/announcements/{id}",
            axum::routing::put(controller::update_announcement)
                .delete(controller::delete_announcement),
        )
}
