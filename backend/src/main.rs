mod models;
mod srs;
mod store;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::{
    models::{IngestProblemInput, NotificationPreference, ProblemStatus, User},
    srs::{Grade, load_schedule},
    store::InMemoryStore,
};

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<InMemoryStore>>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
}

impl ApiError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RequestMagicLink {
    email: String,
}

#[derive(Debug, Serialize)]
struct RequestMagicLinkResponse {
    sent: bool,
    // Returned for MVP testing/dev convenience.
    dev_magic_token: String,
}

#[derive(Debug, Deserialize)]
struct VerifyMagicLink {
    token: String,
}

#[derive(Debug, Serialize)]
struct VerifyMagicLinkResponse {
    session_token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct IngestProblemEventRequest {
    source: String,
    problem_slug: String,
    title: String,
    url: String,
    status: ProblemStatus,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GradeRequest {
    grade: Grade,
}

#[derive(Debug, Serialize)]
struct DashboardResponse {
    due_count: usize,
    upcoming_count: usize,
    leetcode_count: usize,
    neetcode_count: usize,
    latest_ingestion: Option<crate::models::ProblemEvent>,
}

#[derive(Debug, Deserialize)]
struct SaveSettingsRequest {
    email_enabled: bool,
    digest_hour_utc: u8,
}

#[derive(Debug, Serialize)]
struct IntegrationsResponse {
    session_token_setup: Vec<String>,
    latest_event: Option<crate::models::ProblemEvent>,
    checklist: Vec<String>,
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.strip_prefix("Bearer "))
}

async fn authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<User, (StatusCode, Json<ApiError>)> {
    let token = bearer_token(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("Missing bearer token")),
        )
    })?;
    let store = state.store.lock().await;
    store.user_from_session(token).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("Invalid session")),
        )
    })
}

async fn request_magic_link(
    State(state): State<AppState>,
    Json(payload): Json<RequestMagicLink>,
) -> Result<Json<RequestMagicLinkResponse>, (StatusCode, Json<ApiError>)> {
    if !payload.email.contains('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Invalid email")),
        ));
    }
    let mut store = state.store.lock().await;
    let user = store.get_or_create_user(&payload.email.to_lowercase());
    let magic_token = store.create_magic_link(user.id);
    Ok(Json(RequestMagicLinkResponse {
        sent: true,
        dev_magic_token: magic_token,
    }))
}

async fn verify_magic_link(
    State(state): State<AppState>,
    Json(payload): Json<VerifyMagicLink>,
) -> Result<Json<VerifyMagicLinkResponse>, (StatusCode, Json<ApiError>)> {
    let mut store = state.store.lock().await;
    let Some((user, session_token)) = store.verify_magic_link(&payload.token) else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("Invalid or expired magic link")),
        ));
    };
    Ok(Json(VerifyMagicLinkResponse {
        session_token,
        user,
    }))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let token = bearer_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("Missing bearer token")),
        )
    })?;
    let mut store = state.store.lock().await;
    store.revoke_session(token);
    Ok(StatusCode::NO_CONTENT)
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    Ok(Json(user))
}

async fn ingest_problem_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<IngestProblemEventRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let mut store = state.store.lock().await;
    let event = store.ingest_event(IngestProblemInput {
        user_id: user.id,
        source: payload.source.to_lowercase(),
        problem_slug: payload.problem_slug,
        title: payload.title,
        url: payload.url,
        status: payload.status,
        occurred_at: payload.occurred_at,
    });
    Ok((StatusCode::CREATED, Json(event)))
}

async fn due_reviews(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::models::ProblemCard>>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let store = state.store.lock().await;
    Ok(Json(store.due_cards(user.id, Utc::now())))
}

async fn grade_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(card_id): Path<i64>,
    Json(payload): Json<GradeRequest>,
) -> Result<Json<crate::models::ReviewEvent>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let mut store = state.store.lock().await;
    let Some(review) = store.grade_card(user.id, card_id, payload.grade) else {
        return Err((StatusCode::NOT_FOUND, Json(ApiError::new("Card not found"))));
    };
    Ok(Json(review))
}

async fn history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::models::ReviewEvent>>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let store = state.store.lock().await;
    Ok(Json(store.user_history(user.id)))
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DashboardResponse>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let store = state.store.lock().await;
    let due = store.due_cards(user.id, Utc::now());
    let upcoming = store.upcoming_cards(user.id);
    let leetcode_count = upcoming
        .iter()
        .filter(|card| card.source == "leetcode")
        .count();
    let neetcode_count = upcoming
        .iter()
        .filter(|card| card.source == "neetcode")
        .count();
    Ok(Json(DashboardResponse {
        due_count: due.len(),
        upcoming_count: upcoming.len(),
        leetcode_count,
        neetcode_count,
        latest_ingestion: store.latest_event_for_user(user.id),
    }))
}

async fn get_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<NotificationPreference>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let store = state.store.lock().await;
    let Some(pref) = store.get_notification_preference(user.id) else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Settings not found")),
        ));
    };
    Ok(Json(pref))
}

async fn save_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SaveSettingsRequest>,
) -> Result<Json<NotificationPreference>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let mut store = state.store.lock().await;
    let Some(pref) =
        store.set_notification_preference(user.id, payload.email_enabled, payload.digest_hour_utc)
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Settings not found")),
        ));
    };
    Ok(Json(pref))
}

async fn integrations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<IntegrationsResponse>, (StatusCode, Json<ApiError>)> {
    let user = authenticated_user(&state, &headers).await?;
    let store = state.store.lock().await;
    Ok(Json(IntegrationsResponse {
        session_token_setup: vec![
            "Use the real app session token (not a placeholder)".to_owned(),
            "After login + verify, copy localStorage.srs_session_token from the SRS app tab"
                .to_owned(),
            "Set the same value in LeetCode/NeetCode localStorage.srs_session_token".to_owned(),
        ],
        latest_event: store.latest_event_for_user(user.id),
        checklist: vec![
            "Install Tampermonkey extension".to_owned(),
            "Install script from integrations page".to_owned(),
            "Login with your LeetCode account".to_owned(),
            "Open or solve a problem and verify the event appears here".to_owned(),
        ],
    }))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/request-magic-link", post(request_magic_link))
        .route("/auth/verify-magic-link", post(verify_magic_link))
        .route("/auth/logout", post(logout))
        .route("/me", get(me))
        .route("/events/problem-status", post(ingest_problem_event))
        .route("/reviews/due", get(due_reviews))
        .route("/reviews/{card_id}/grade", post(grade_review))
        .route("/history", get(history))
        .route("/dashboard", get(dashboard))
        .route("/settings", get(get_settings).post(save_settings))
        .route("/integrations", get(integrations))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn email_digest_worker(state: AppState) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let now = Utc::now();
        let mut store = state.store.lock().await;
        let users: Vec<User> = store.users.values().cloned().collect();
        for user in users {
            let Some(pref) = store.get_notification_preference(user.id) else {
                continue;
            };
            if !pref.email_enabled || pref.digest_hour_utc != now.hour() as u8 {
                continue;
            }
            let due = store.due_cards(user.id, now);
            if due.is_empty() {
                continue;
            }
            let subject = format!("SRS reminder: {} reviews due", due.len());
            let body = format!(
                "You have {} due reviews. Open your dashboard to continue.",
                due.len()
            );
            store.log_email(user.id, &subject, &body);
            info!("queued email digest for user {}", user.id);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let schedule = load_schedule(
        std::env::var("SRS_CONFIG_PATH").ok().as_deref(),
        std::env::var("SRS_PROFILE").ok().as_deref(),
    );
    let state = AppState {
        store: Arc::new(Mutex::new(InMemoryStore::new_with_schedule(schedule))),
    };
    tokio::spawn(email_digest_worker(state.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind server");
    info!("backend listening on http://0.0.0.0:3000");
    axum::serve(listener, app(state)).await.expect("serve app");
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::IntegrationsResponse;

    #[test]
    fn integrations_response_uses_setup_steps_not_hint_token() {
        let response = IntegrationsResponse {
            session_token_setup: vec![
                "Use the real app session token (not a placeholder)".to_owned(),
                "Copy localStorage.srs_session_token from app auth flow".to_owned(),
            ],
            latest_event: None,
            checklist: vec!["Install Tampermonkey extension".to_owned()],
        };

        let payload = serde_json::to_value(&response).expect("serialize integrations response");
        let object = payload
            .as_object()
            .expect("integrations response should serialize to an object");

        assert!(object.contains_key("session_token_setup"));
        assert!(!object.contains_key("ingestion_token_hint"));
        assert_eq!(object.get("latest_event"), Some(&Value::Null));
    }
}
