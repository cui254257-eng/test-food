use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct WeChatLoginInput {
    pub(crate) code: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AdminLoginInput {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminLoginResponse {
    pub(crate) token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoginResponse {
    pub(crate) token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SessionResponse {
    pub(crate) authenticated: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateProfileInput {
    pub(crate) nickname: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProfileResponse {
    pub(crate) nickname: String,
    pub(crate) avatar_data_url: Option<String>,
    pub(crate) member_level: i64,
    pub(crate) points: i64,
    pub(crate) discount_rate: f64,
    pub(crate) next_level: Option<i64>,
    pub(crate) next_level_points: Option<i64>,
    pub(crate) points_to_next_level: Option<i64>,
}
