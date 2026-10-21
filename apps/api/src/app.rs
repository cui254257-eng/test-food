use std::path::PathBuf;

use axum::{
    Json, Router,
    http::{HeaderMap, header::AUTHORIZATION},
    routing::get,
};
use serde_json::json;
use sqlx::MySqlPool;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::modules::{
    addresses::{AddressesService, routes as address_routes},
    auth::{AuthService, routes as auth_routes},
    banners::{BannersService, routes as banner_routes},
    home_features::{HomeFeaturesService, routes as home_feature_routes},
    membership::{MembershipService, routes as membership_routes},
    orders::{OrdersService, routes as order_routes},
    products::{ProductsService, routes as product_routes},
    restaurant_settings::{RestaurantSettingsService, routes as restaurant_settings_routes},
};

/// Application-wide dependencies shared with request handlers.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) products: ProductsService,
    pub(crate) banners: BannersService,
    pub(crate) home_features: HomeFeaturesService,
    pub(crate) auth: AuthService,
    pub(crate) addresses: AddressesService,
    pub(crate) restaurant_settings: RestaurantSettingsService,
    pub(crate) orders: OrdersService,
    pub(crate) membership: MembershipService,
    pub(crate) admin_api_token: Option<String>,
    pub(crate) admin_username: String,
    pub(crate) admin_password: String,
}

impl AppState {
    pub(crate) fn authenticate_admin(
        &self,
        headers: &HeaderMap,
    ) -> Result<(), crate::common::ApiError> {
        let Some(admin_token) = self.admin_api_token.as_deref() else {
            return Err(crate::common::ApiError::service_unavailable(
                "管理员登录暂未启用，请检查 API 配置",
            ));
        };
        let supplied_token = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if !supplied_token.is_some_and(|token| token == admin_token) {
            return Err(crate::common::ApiError::unauthorized(
                "管理员凭证无效或已过期",
            ));
        }
        Ok(())
    }

    pub(crate) fn login_admin(
        &self,
        username: &str,
        password: &str,
    ) -> Result<String, crate::common::ApiError> {
        if username != self.admin_username || password != self.admin_password {
            return Err(crate::common::ApiError::unauthorized("账号或密码错误"));
        }
        self.admin_api_token.clone().ok_or_else(|| {
            crate::common::ApiError::service_unavailable("管理员登录暂未启用，请检查 API 配置")
        })
    }
}

/// Builds the HTTP application and wires feature modules together.
pub(crate) fn create_router(
    pool: MySqlPool,
    wechat_app_id: String,
    wechat_app_secret: String,
    admin_api_token: Option<String>,
    admin_username: String,
    admin_password: String,
) -> Router {
    let state = AppState {
        products: ProductsService::new(pool.clone()),
        banners: BannersService::new(pool.clone()),
        home_features: HomeFeaturesService::new(pool.clone()),
        auth: AuthService::new(
            // Credentials are server-side only; an empty value disables login clearly.
            pool.clone(),
            wechat_app_id,
            wechat_app_secret,
        ),
        addresses: AddressesService::new(pool.clone()),
        restaurant_settings: RestaurantSettingsService::new(pool.clone()),
        orders: OrdersService::new(pool.clone()),
        membership: MembershipService::new(pool),
        admin_api_token,
        admin_username,
        admin_password,
    };
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    Router::new()
        .route("/health", get(health))
        .merge(product_routes())
        .merge(banner_routes())
        .merge(home_feature_routes())
        .merge(auth_routes())
        .merge(address_routes())
        .merge(restaurant_settings_routes())
        .merge(order_routes())
        .merge(membership_routes())
        .nest_service("/assets", ServeDir::new(assets_dir))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
