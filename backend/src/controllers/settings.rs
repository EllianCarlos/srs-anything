use axum::{Json, extract::State, http::StatusCode};
use tracing::info;

use crate::{
    app::state::AppState, dto::settings::SaveSettingsRequest,
    extractors::authenticated_user::AuthenticatedUser, models::NotificationPreference,
};

pub async fn get_settings(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<NotificationPreference>, (StatusCode, Json<crate::errors::ApiError>)> {
    let pref = state
        .settings_service
        .get(user.id)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        email_enabled = pref.email_enabled,
        digest_hour_utc = pref.digest_hour_utc,
        "settings_get"
    );
    Ok(Json(pref))
}

pub async fn save_settings(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<SaveSettingsRequest>,
) -> Result<Json<NotificationPreference>, (StatusCode, Json<crate::errors::ApiError>)> {
    info!(
        email_enabled = payload.email_enabled,
        digest_hour_utc = payload.digest_hour_utc,
        "settings_save_request"
    );
    let pref = state
        .settings_service
        .save(user.id, payload.email_enabled, payload.digest_hour_utc)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        email_enabled = pref.email_enabled,
        digest_hour_utc = pref.digest_hour_utc,
        "settings_saved"
    );
    Ok(Json(pref))
}
