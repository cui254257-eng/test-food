use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/orders",
            get(controller::list_customer_orders).post(controller::create),
        )
        .route("/api/admin/orders", get(controller::list_admin_orders))
        .route(
            "/api/admin/orders/{order_number}",
            axum::routing::delete(controller::delete_admin_order),
        )
        .route(
            "/api/admin/orders/{order_number}/paid",
            axum::routing::post(controller::mark_admin_order_paid),
        )
}
