use std::time::Duration;

use axum::{
    Router,
    extract::MatchedPath,
    http::{HeaderValue, Method, Request, header},
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    request_id::{PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultOnFailure, TraceLayer},
};
use tracing::{Span, field};

use crate::{
    app::state::AppState,
    controllers::{
        auth::{logout, me, request_magic_link, verify_magic_link},
        dashboard::dashboard,
        events::ingest_problem_event,
        integrations::{create_integration_token, integrations, revoke_integration_token},
        reviews::{due_reviews, grade_review, history},
        settings::{get_settings, save_settings},
    },
};

pub fn app_router(state: AppState) -> Router {
    let allowed_origins = state
        .security
        .allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();

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
        .route("/integrations/tokens", post(create_integration_token))
        .route(
            "/integrations/tokens/{token_id}",
            axum::routing::delete(revoke_integration_token),
        )
        .with_state(state)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(GenerateRequestId))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::COOKIE,
                    header::HeaderName::from_static("x-api-key"),
                ]),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|request_id| request_id.header_value().to_str().ok())
                        .unwrap_or("unknown");
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or(request.uri().path());

                    tracing::info_span!(
                        "http_request",
                        requestId = %request_id,
                        method = %request.method(),
                        path = matched_path,
                        status = field::Empty,
                        latency_ms = field::Empty
                    )
                })
                .on_request(|_: &Request<_>, _: &Span| {
                    tracing::info!("request_started");
                })
                .on_response(
                    |response: &axum::response::Response, latency: Duration, span: &Span| {
                        span.record("status", response.status().as_u16());
                        span.record("latency_ms", latency.as_millis() as u64);
                        tracing::info!("request_finished");
                    },
                )
                .on_failure(DefaultOnFailure::new().level(tracing::Level::WARN)),
        )
}

#[derive(Clone, Copy, Debug, Default)]
struct GenerateRequestId;

impl tower_http::request_id::MakeRequestId for GenerateRequestId {
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
        if let Some(existing_id) = request.headers().get("x-request-id") {
            return Some(RequestId::new(existing_id.clone()));
        }

        let generated = format!("{:032x}", rand::random::<u128>());
        HeaderValue::from_str(&generated).ok().map(RequestId::new)
    }
}
