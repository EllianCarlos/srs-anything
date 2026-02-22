use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{app::state::AppState, controllers::api_key, models::User};

#[derive(Debug, Clone)]
pub struct ApiKeyUser(pub User);

impl<S> FromRequestParts<S> for ApiKeyUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (axum::http::StatusCode, axum::Json<crate::errors::ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = app_state
            .integrations_service
            .user_from_api_key(api_key(&parts.headers))
            .await
            .map_err(|err| err.to_http())?;
        Ok(Self(user))
    }
}
