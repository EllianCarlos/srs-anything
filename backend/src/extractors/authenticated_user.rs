use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, Method, request::Parts},
};

use crate::{app::state::AppState, controllers::auth_cookie, errors::AppError, models::User};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (axum::http::StatusCode, axum::Json<crate::errors::ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        if is_mutating(parts.method.clone()) {
            validate_origin(&parts.headers, &app_state.security.allowed_origins)
                .map_err(|err| err.to_http())?;
        }
        let user = app_state
            .auth_service
            .user_from_jwt_cookie(auth_cookie(&parts.headers))
            .await
            .map_err(|err| err.to_http())?;
        Ok(Self(user))
    }
}

fn is_mutating(method: Method) -> bool {
    !matches!(
        method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

fn validate_origin(
    headers: &HeaderMap,
    allowed_origins: &std::collections::HashSet<String>,
) -> Result<(), AppError> {
    let origin_or_referer = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .or_else(|| headers.get("referer").and_then(|value| value.to_str().ok()))
        .ok_or(AppError::Forbidden)?;
    if allowed_origins
        .iter()
        .any(|allowed| origin_or_referer.starts_with(allowed))
    {
        return Ok(());
    }
    Err(AppError::Forbidden)
}
