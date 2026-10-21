use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct WeChatLoginInput {
    pub(crate) code: String,
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
}
