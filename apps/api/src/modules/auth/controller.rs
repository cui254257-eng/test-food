use axum::{
    Json,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
};

use crate::{app::AppState, common::ApiError};

use super::dto::{
    LoginResponse, ProfileResponse, SessionResponse, UpdateProfileInput, WeChatLoginInput,
};

pub(crate) async fn wechat_login(
    State(state): State<AppState>,
    Json(input): Json<WeChatLoginInput>,
) -> Result<Json<LoginResponse>, ApiError> {
    let token = state.auth.login_with_wechat_code(&input.code).await?;
    Ok(Json(LoginResponse { token }))
}

pub(crate) async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    state.auth.authenticate(&headers).await?;
    Ok(Json(SessionResponse {
        authenticated: true,
    }))
}

pub(crate) async fn profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ProfileResponse>, ApiError> {
    Ok(Json(state.auth.get_profile(&headers).await?))
}

pub(crate) async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<UpdateProfileInput>,
) -> Result<Json<ProfileResponse>, ApiError> {
    Ok(Json(
        state
            .auth
            .update_nickname(&headers, &input.nickname)
            .await?,
    ))
}

pub(crate) async fn update_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ProfileResponse>, ApiError> {
    let mut avatar = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("头像上传数据无效"))?
    {
        if field.name() == Some("avatar") {
            avatar = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| ApiError::bad_request("头像文件读取失败"))?
                    .to_vec(),
            );
            break;
        }
    }
    let avatar = avatar.ok_or_else(|| ApiError::bad_request("请选择头像文件"))?;
    Ok(Json(state.auth.update_avatar(&headers, &avatar).await?))
}

pub(crate) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    state.auth.logout(&headers).await?;
    Ok(StatusCode::NO_CONTENT)
}
