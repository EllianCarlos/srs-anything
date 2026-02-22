use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use tracing::info;

use crate::{
    app::state::AppState, dto::events::IngestProblemEventRequest,
    extractors::api_key_user::ApiKeyUser, models::IngestProblemInput,
};

pub async fn ingest_problem_event(
    State(state): State<AppState>,
    ApiKeyUser(user): ApiKeyUser,
    Json(payload): Json<IngestProblemEventRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<crate::errors::ApiError>)> {
    info!(
        source = %payload.source,
        problem_slug = %payload.problem_slug,
        "events_ingest_request"
    );
    let event = state
        .event_service
        .ingest(IngestProblemInput {
            user_id: user.id,
            source: payload.source.to_lowercase(),
            problem_slug: payload.problem_slug,
            title: payload.title,
            url: payload.url,
            status: payload.status,
            occurred_at: payload.occurred_at,
        })
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        event_id = event.id,
        source = %event.source,
        problem_slug = %event.problem_slug,
        "events_ingest_created"
    );
    Ok((StatusCode::CREATED, Json(event)))
}
