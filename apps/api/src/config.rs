use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use std::env;

pub(crate) struct Settings {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) database_url: String,
    // Keep these server-only values out of the mini-program bundle.
    pub(crate) wechat_app_id: String,
    pub(crate) wechat_app_secret: String,
    // Required to read all customers' orders from the management console.
    pub(crate) admin_api_token: Option<String>,
    pub(crate) admin_username: String,
    pub(crate) admin_password: String,
}

impl Settings {
    pub(crate) fn from_env() -> Result<Self, std::num::ParseIntError> {
        let port = env::var("API_PORT")
            .unwrap_or_else(|_| "8000".into())
            .parse()?;

        let configured_admin_api_token = env::var("ADMIN_API_TOKEN")
            .ok()
            .map(|token| token.trim().to_owned())
            .filter(|token| !token.is_empty());
        let admin_api_token = match configured_admin_api_token {
            Some(token) => Some(token),
            None => {
                tracing::warn!(
                    "ADMIN_API_TOKEN is not set; admin tokens will change whenever the API process restarts"
                );
                // Use an unpredictable process-local token when one is not configured.
                let mut bytes = [0u8; 32];
                OsRng.fill_bytes(&mut bytes);
                Some(URL_SAFE_NO_PAD.encode(bytes))
            }
        };

        Ok(Self {
            host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3307/shop".into()),
            wechat_app_id: env::var("WECHAT_APP_ID").unwrap_or_default(),
            wechat_app_secret: env::var("WECHAT_APP_SECRET").unwrap_or_default(),
            admin_api_token,
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".into()),
        })
    }
}
