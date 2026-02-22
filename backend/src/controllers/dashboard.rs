use axum::{Json, extract::State, http::StatusCode};
use tracing::info;

use crate::{
    app::state::AppState, dto::dashboard::DashboardResponse,
    extractors::authenticated_user::AuthenticatedUser,
};

pub async fn dashboard(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<DashboardResponse>, (StatusCode, Json<crate::errors::ApiError>)> {
    let dashboard = state
        .dashboard_service
        .dashboard_for_user(user.id)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        due_count = dashboard.due_count,
        upcoming_count = dashboard.upcoming_count,
        "dashboard_get"
    );
    Ok(Json(dashboard))
}
