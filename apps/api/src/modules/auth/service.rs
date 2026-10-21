use axum::http::HeaderMap;
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use rand::{RngCore, rngs::OsRng};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, MySqlPool};

use crate::common::ApiError;

use super::dto::ProfileResponse;

const WECHAT_SESSION_URL: &str = "https://api.weixin.qq.com/sns/jscode2session";
const SESSION_LIFETIME_DAYS: i64 = 7;
const MAX_AVATAR_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub(crate) struct AuthService {
    pool: MySqlPool,
    app_id: String,
    app_secret: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct WeChatSessionResponse {
    openid: Option<String>,
    errcode: Option<i64>,
}

#[derive(Debug, FromRow)]
struct CustomerProfileRecord {
    nickname: String,
    avatar_mime_type: Option<String>,
    avatar_base64: Option<String>,
}

impl AuthService {
    pub(crate) fn new(pool: MySqlPool, app_id: String, app_secret: String) -> Self {
        Self {
            pool,
            app_id,
            app_secret,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client configuration is valid"),
        }
    }

    pub(crate) async fn login_with_wechat_code(&self, code: &str) -> Result<String, ApiError> {
        if self.app_id.trim().is_empty() || self.app_secret.trim().is_empty() {
            return Err(ApiError::service_unavailable(
                "服务端尚未配置微信小程序 AppID 和 Secret",
            ));
        }
        if code.trim().is_empty() || code.len() > 512 {
            return Err(ApiError::bad_request("微信登录凭证无效，请重新点击登录"));
        }

        // Exchange the fresh uni.login code only on the server; never send AppSecret to the client.
        let response = self
            .http
            .get(WECHAT_SESSION_URL)
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|_| ApiError::bad_gateway("暂时无法连接微信登录服务，请稍后重试"))?
            .json::<WeChatSessionResponse>()
            .await
            .map_err(|_| ApiError::bad_gateway("微信登录服务返回了无法识别的结果"))?;

        if let Some(error_code) = response.errcode.filter(|code| *code != 0) {
            return Err(match error_code {
                40029 => ApiError::bad_request("微信登录凭证已失效，请重新点击“微信登录”后再试"),
                40013 => ApiError::service_unavailable(
                    "微信 AppID 无效，请检查服务端 WECHAT_APP_ID 配置",
                ),
                40125 => ApiError::service_unavailable(
                    "微信 AppSecret 无效，请检查服务端 WECHAT_APP_SECRET 配置",
                ),
                code => ApiError::bad_gateway(format!(
                    "微信登录服务返回错误码 {code}，请检查 AppID/Secret 配置"
                )),
            });
        }
        let openid = response
            .openid
            .filter(|openid| !openid.is_empty())
            .ok_or_else(|| ApiError::bad_gateway("微信登录服务未返回用户标识"))?;

        self.issue_session(&openid).await
    }

    pub(crate) async fn authenticate(&self, headers: &HeaderMap) -> Result<i64, ApiError> {
        let token = bearer_token(headers).ok_or_else(|| ApiError::unauthorized("请先微信登录"))?;
        let token_hash = hash_token(token);

        sqlx::query_scalar::<_, i64>(
            "SELECT customer_id FROM customer_sessions \
             WHERE token_hash = ? AND expires_at > CURRENT_TIMESTAMP(3)",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::unauthorized("登录已过期，请重新登录"))
    }

    pub(crate) async fn get_profile(
        &self,
        headers: &HeaderMap,
    ) -> Result<ProfileResponse, ApiError> {
        let customer_id = self.authenticate(headers).await?;
        self.profile_by_customer_id(customer_id).await
    }

    pub(crate) async fn update_nickname(
        &self,
        headers: &HeaderMap,
        nickname: &str,
    ) -> Result<ProfileResponse, ApiError> {
        let customer_id = self.authenticate(headers).await?;
        let nickname = nickname.trim();
        let character_count = nickname.chars().count();
        if character_count == 0 || character_count > 32 || nickname.chars().any(char::is_control) {
            return Err(ApiError::bad_request(
                "昵称需为 1 到 32 个字符，且不能包含控制字符",
            ));
        }

        sqlx::query("UPDATE customers SET nickname = ? WHERE id = ?")
            .bind(nickname)
            .bind(customer_id)
            .execute(&self.pool)
            .await?;

        self.profile_by_customer_id(customer_id).await
    }

    pub(crate) async fn update_avatar(
        &self,
        headers: &HeaderMap,
        image: &[u8],
    ) -> Result<ProfileResponse, ApiError> {
        let customer_id = self.authenticate(headers).await?;
        if image.is_empty() || image.len() > MAX_AVATAR_BYTES {
            return Err(ApiError::bad_request("头像文件需小于 1 MB"));
        }
        let mime_type = detect_avatar_mime_type(image)
            .ok_or_else(|| ApiError::bad_request("头像仅支持 JPEG、PNG 或 WebP 图片"))?;
        let encoded = STANDARD.encode(image);

        sqlx::query("UPDATE customers SET avatar_mime_type = ?, avatar_base64 = ? WHERE id = ?")
            .bind(mime_type)
            .bind(encoded)
            .bind(customer_id)
            .execute(&self.pool)
            .await?;

        self.profile_by_customer_id(customer_id).await
    }

    pub(crate) async fn logout(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        if let Some(token) = bearer_token(headers) {
            sqlx::query("DELETE FROM customer_sessions WHERE token_hash = ?")
                .bind(hash_token(token))
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn profile_by_customer_id(&self, customer_id: i64) -> Result<ProfileResponse, ApiError> {
        let profile = sqlx::query_as::<_, CustomerProfileRecord>(
            "SELECT nickname, avatar_mime_type, avatar_base64 FROM customers WHERE id = ?",
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::not_found("顾客资料不存在"))?;

        let avatar_data_url = profile
            .avatar_mime_type
            .zip(profile.avatar_base64)
            .map(|(mime_type, data)| format!("data:{mime_type};base64,{data}"));
        Ok(ProfileResponse {
            nickname: profile.nickname,
            avatar_data_url,
        })
    }

    async fn issue_session(&self, openid: &str) -> Result<String, ApiError> {
        // Reuse this mini-program's account by OpenID, or register it on its first login.
        let customer = sqlx::query(
            "INSERT INTO customers (openid, nickname) VALUES (?, '') \
             ON DUPLICATE KEY UPDATE id = LAST_INSERT_ID(id)",
        )
        .bind(openid)
        .execute(&self.pool)
        .await?;
        let customer_id = customer.last_insert_id() as i64;
        if customer_id < 1 {
            return Err(ApiError::bad_gateway("无法创建小程序用户会话"));
        }
        let default_nickname = format!("微信用户{:06}", OsRng.next_u32() % 1_000_000);
        sqlx::query("UPDATE customers SET nickname = ? WHERE id = ? AND TRIM(nickname) = ''")
            .bind(default_nickname)
            .bind(customer_id)
            .execute(&self.pool)
            .await?;

        let mut random = [0u8; 32];
        OsRng
            .try_fill_bytes(&mut random)
            .map_err(|_| ApiError::bad_gateway("无法创建安全登录凭证"))?;
        let token = URL_SAFE_NO_PAD.encode(random);
        let token_hash = hash_token(&token);

        // Remove expired sessions during login to keep the session table bounded.
        sqlx::query("DELETE FROM customer_sessions WHERE expires_at <= CURRENT_TIMESTAMP(3)")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "INSERT INTO customer_sessions (token_hash, customer_id, expires_at) \
             VALUES (?, ?, TIMESTAMPADD(DAY, ?, CURRENT_TIMESTAMP(3)))",
        )
        .bind(token_hash)
        .bind(customer_id)
        .bind(SESSION_LIFETIME_DAYS)
        .execute(&self.pool)
        .await?;

        Ok(token)
    }
}

fn detect_avatar_mime_type(image: &[u8]) -> Option<&'static str> {
    if image.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if image.starts_with(b"\x89PNG\r\n\x1A\n") {
        Some("image/png")
    } else if image.len() >= 12 && &image[..4] == b"RIFF" && &image[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
