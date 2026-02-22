use axum::{
    Json,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use tracing::info;

use crate::{
    app::state::AppState,
    dto::auth::{
        RequestMagicLink, RequestMagicLinkResponse, VerifyMagicLink, VerifyMagicLinkResponse,
    },
    extractors::authenticated_user::AuthenticatedUser,
};

pub async fn request_magic_link(
    State(state): State<AppState>,
    Json(payload): Json<RequestMagicLink>,
) -> Result<Json<RequestMagicLinkResponse>, (StatusCode, Json<crate::errors::ApiError>)> {
    info!(email = %payload.email, "auth_request_magic_link");
    let token = state
        .auth_service
        .request_magic_link(&payload.email)
        .await
        .map_err(|err| err.to_http())?;
    Ok(Json(RequestMagicLinkResponse {
        sent: true,
        dev_magic_token: token,
    }))
}

pub async fn verify_magic_link(
    State(state): State<AppState>,
    Json(payload): Json<VerifyMagicLink>,
) -> Result<impl IntoResponse, (StatusCode, Json<crate::errors::ApiError>)> {
    info!("auth_verify_magic_link");
    let (user, jwt) = state
        .auth_service
        .verify_magic_link(&payload.token)
        .await
        .map_err(|err| err.to_http())?;
    let cookie = build_auth_cookie(
        &jwt,
        state.auth_service.session_max_age_secs(),
        state.security.cookie_secure,
    )?;
    info!(user_id = user.id, "auth_verify_magic_link_success");
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(VerifyMagicLinkResponse { user }),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, Json<crate::errors::ApiError>)> {
    info!("auth_logout");
    let clear_cookie = clear_auth_cookie(state.security.cookie_secure)?;
    info!(user_id = user.id, "auth_logout_success");
    Ok(([(header::SET_COOKIE, clear_cookie)], StatusCode::NO_CONTENT))
}

pub async fn me(
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<crate::models::User>, (StatusCode, Json<crate::errors::ApiError>)> {
    info!(user_id = user.id, "auth_me");
    Ok(Json(user))
}

fn build_auth_cookie(
    jwt: &str,
    max_age_secs: i64,
    secure: bool,
) -> Result<HeaderValue, (StatusCode, Json<crate::errors::ApiError>)> {
    let secure_part = if secure { "; Secure" } else { "" };
    let value = format!(
        "srs_auth={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}{secure_part}"
    );
    HeaderValue::from_str(&value).map_err(|err| {
        crate::errors::AppError::Internal(format!("invalid cookie header: {err}")).to_http()
    })
}

fn clear_auth_cookie(
    secure: bool,
) -> Result<HeaderValue, (StatusCode, Json<crate::errors::ApiError>)> {
    let secure_part = if secure { "; Secure" } else { "" };
    let value = format!("srs_auth=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_part}");
    HeaderValue::from_str(&value).map_err(|err| {
        crate::errors::AppError::Internal(format!("invalid clear-cookie header: {err}")).to_http()
    })
}
