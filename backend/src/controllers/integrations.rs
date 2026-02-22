use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::info;

use crate::{
    app::state::AppState,
    dto::integrations::{
        CreateIntegrationTokenRequest, CreateIntegrationTokenResponse, IntegrationsResponse,
    },
    extractors::authenticated_user::AuthenticatedUser,
};

pub async fn integrations(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<IntegrationsResponse>, (StatusCode, Json<crate::errors::ApiError>)> {
    let response = state
        .integrations_service
        .get(user.id)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        has_latest_event = response.latest_event.is_some(),
        "integrations_get"
    );
    Ok(Json(response))
}

pub async fn create_integration_token(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateIntegrationTokenRequest>,
) -> Result<Json<CreateIntegrationTokenResponse>, (StatusCode, Json<crate::errors::ApiError>)> {
    let created = state
        .integrations_service
        .create_token(user.id, &payload.label, payload.expires_in_days)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        token_id = created.token_summary.id,
        "integrations_token_created"
    );
    Ok(Json(created))
}

pub async fn revoke_integration_token(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(token_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<crate::errors::ApiError>)> {
    state
        .integrations_service
        .revoke_token(user.id, token_id)
        .await
        .map_err(|err| err.to_http())?;
    info!(user_id = user.id, token_id, "integrations_token_revoked");
    Ok(StatusCode::NO_CONTENT)
}
