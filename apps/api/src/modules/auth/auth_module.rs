use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/wechat", post(controller::wechat_login))
        .route("/api/auth/session", get(controller::session))
        .route(
            "/api/auth/profile",
            get(controller::profile).put(controller::update_profile),
        )
        .route(
            "/api/auth/profile/avatar",
            post(controller::update_avatar).layer(DefaultBodyLimit::max(2 * 1024 * 1024)),
        )
        .route("/api/auth/logout", post(controller::logout))
}
