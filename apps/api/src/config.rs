use std::env;

pub(crate) struct Settings {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) database_url: String,
    // Keep these server-only values out of the mini-program bundle.
    pub(crate) wechat_app_id: String,
    pub(crate) wechat_app_secret: String,
}

impl Settings {
    pub(crate) fn from_env() -> Result<Self, std::num::ParseIntError> {
        let port = env::var("API_PORT")
            .unwrap_or_else(|_| "8000".into())
            .parse()?;

        Ok(Self {
            host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3307/shop".into()),
            wechat_app_id: env::var("WECHAT_APP_ID").unwrap_or_default(),
            wechat_app_secret: env::var("WECHAT_APP_SECRET").unwrap_or_default(),
        })
    }
}
