mod app;
mod common;
mod config;
mod database;
mod modules;

use std::{net::SocketAddr, time::Duration};

use tracing_subscriber::EnvFilter;

use crate::{app::create_router, config::Settings, database::connect};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    init_tracing();

    let settings = Settings::from_env()?;
    let pool = connect(&settings.database_url).await?;
    let expiration_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            match modules::orders::OrdersService::expire_pending_orders(&expiration_pool).await {
                Ok(count) if count > 0 => {
                    tracing::info!(count, "Expired unpaid orders and released inventory")
                }
                Ok(_) => {}
                Err(error) => tracing::error!(%error, "Failed to expire unpaid orders"),
            }
        }
    });
    let app = create_router(
        pool,
        settings.wechat_app_id,
        settings.wechat_app_secret,
        settings.admin_api_token,
        settings.admin_username,
        settings.admin_password,
    );
    let address = SocketAddr::new(settings.host.parse()?, settings.port);
    let listener = tokio::net::TcpListener::bind(address).await?;

    tracing::info!("Shop API listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
