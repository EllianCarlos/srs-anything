use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use serial_test::serial;
use tower::ServiceExt;

use srs_anything_backend::{
    app::{bootstrap::build_state_with_schedule, routes::app_router},
    srs::{IntervalUnit, ScheduleProfile, SrsSchedule},
};

mod support;

async fn test_app() -> Option<axum::Router> {
    let schedule = SrsSchedule::from_profile(ScheduleProfile {
        unit: IntervalUnit::Minutes,
        intervals: vec![1, 3, 5],
    })
    .expect("valid test schedule");
    let db = support::db::try_test_db().await?;
    support::db::reset_db(&db).await;
    Some(app_router(build_state_with_schedule(schedule, db)))
}

async fn json_response(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json body")
}

fn auth_cookie_from_headers(headers: &axum::http::HeaderMap) -> String {
    let set_cookie = headers
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("set-cookie header");
    set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_owned()
}

#[tokio::test]
#[serial]
async fn auth_lifecycle_works_end_to_end() {
    let Some(app) = test_app().await else {
        return;
    };

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health response");
    assert_eq!(health.status(), StatusCode::OK);

    let request_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/request-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"email":"e2e@test.com"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("request magic link response");
    assert_eq!(request_token.status(), StatusCode::OK);
    let token_body = json_response(request_token).await;
    let magic_token = token_body["dev_magic_token"]
        .as_str()
        .expect("magic token")
        .to_owned();

    let verify = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/verify-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"token":magic_token}).to_string()))
                .expect("request"),
        )
        .await
        .expect("verify response");
    assert_eq!(verify.status(), StatusCode::OK);
    let auth_cookie = auth_cookie_from_headers(verify.headers());
    let verify_body = json_response(verify).await;
    assert!(verify_body.get("session_token").is_none());

    let me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/me")
                .header(header::COOKIE, &auth_cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("me response");
    assert_eq!(me.status(), StatusCode::OK);
    let me_body = json_response(me).await;
    assert_eq!(me_body["email"], "e2e@test.com");

    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header(header::COOKIE, &auth_cookie)
                .header(header::ORIGIN, "http://localhost:5173")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("logout response");
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    let cleared_cookie = auth_cookie_from_headers(logout.headers());

    let me_after_logout = app
        .oneshot(
            Request::builder()
                .uri("/me")
                .header(header::COOKIE, &cleared_cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("me response");
    assert_eq!(me_after_logout.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn request_id_is_set_and_propagated() {
    let Some(app) = test_app().await else {
        return;
    };

    let generated_id_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health response");
    assert_eq!(generated_id_response.status(), StatusCode::OK);
    let generated_id = generated_id_response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok());
    assert!(generated_id.is_some_and(|value| !value.is_empty()));

    let provided_request_id = "e2e-request-id-123";
    let propagated_id_response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("x-request-id", provided_request_id)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health response");
    assert_eq!(propagated_id_response.status(), StatusCode::OK);
    assert_eq!(
        propagated_id_response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok()),
        Some(provided_request_id)
    );
}

#[tokio::test]
#[serial]
async fn ingestion_dedup_and_settings_dashboard_contracts_hold() {
    let Some(app) = test_app().await else {
        return;
    };

    let request_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/request-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"email":"flows@test.com"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("request magic link response");
    let token_body = json_response(request_token).await;
    let magic_token = token_body["dev_magic_token"]
        .as_str()
        .expect("magic token")
        .to_owned();

    let verify = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/verify-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"token":magic_token}).to_string()))
                .expect("request"),
        )
        .await
        .expect("verify response");
    let auth_cookie = auth_cookie_from_headers(verify.headers());

    let create_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/integrations/tokens")
                .header(header::COOKIE, &auth_cookie)
                .header(header::ORIGIN, "http://localhost:5173")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"label":"e2e token","expires_in_days":365}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("create token response");
    assert_eq!(create_token.status(), StatusCode::OK);
    let create_token_body = json_response(create_token).await;
    let api_token = create_token_body["token"]
        .as_str()
        .expect("api token")
        .to_owned();

    let event_payload = json!({
      "source":"leetcode",
      "problem_slug":"two-sum",
      "title":"Two Sum",
      "url":"https://leetcode.com/problems/two-sum",
      "status":"solved",
      "occurred_at":"2026-01-01T00:00:00Z"
    });
    let ingest_with_cookie = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events/problem-status")
                .header(header::COOKIE, &auth_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(event_payload.to_string()))
                .expect("request"),
        )
        .await
        .expect("ingest response");
    assert_eq!(ingest_with_cookie.status(), StatusCode::UNAUTHORIZED);

    let first_ingest = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events/problem-status")
                .header("x-api-key", &api_token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(event_payload.to_string()))
                .expect("request"),
        )
        .await
        .expect("ingest response");
    assert_eq!(first_ingest.status(), StatusCode::CREATED);
    let first_body = json_response(first_ingest).await;

    let second_ingest = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events/problem-status")
                .header("x-api-key", &api_token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(event_payload.to_string()))
                .expect("request"),
        )
        .await
        .expect("ingest response");
    assert_eq!(second_ingest.status(), StatusCode::CREATED);
    let second_body = json_response(second_ingest).await;
    assert_eq!(first_body["id"], second_body["id"]);

    let dashboard = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .header(header::COOKIE, &auth_cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("dashboard response");
    assert_eq!(dashboard.status(), StatusCode::OK);
    let dashboard_body = json_response(dashboard).await;
    assert_eq!(dashboard_body["upcoming_count"], 1);
    assert_eq!(dashboard_body["leetcode_count"], 1);
    assert_eq!(dashboard_body["neetcode_count"], 0);

    let get_settings = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .header(header::COOKIE, &auth_cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("settings response");
    assert_eq!(get_settings.status(), StatusCode::OK);
    let current_settings = json_response(get_settings).await;
    assert_eq!(current_settings["email_enabled"], true);

    let save_settings = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/settings")
                .header(header::COOKIE, &auth_cookie)
                .header(header::ORIGIN, "http://localhost:5173")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email_enabled":false,"digest_hour_utc":15}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("save settings response");
    assert_eq!(save_settings.status(), StatusCode::OK);
    let saved_settings = json_response(save_settings).await;
    assert_eq!(saved_settings["email_enabled"], false);
    assert_eq!(saved_settings["digest_hour_utc"], 15);
}

#[tokio::test]
#[serial]
async fn csrf_rejects_mutating_cookie_requests_without_origin() {
    let Some(app) = test_app().await else {
        return;
    };

    let request_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/request-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"email":"csrf@test.com"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("request magic link response");
    let token_body = json_response(request_token).await;
    let magic_token = token_body["dev_magic_token"]
        .as_str()
        .expect("magic token")
        .to_owned();

    let verify = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/verify-magic-link")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"token":magic_token}).to_string()))
                .expect("request"),
        )
        .await
        .expect("verify response");
    assert_eq!(verify.status(), StatusCode::OK);
    let auth_cookie = auth_cookie_from_headers(verify.headers());

    let save_settings = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/settings")
                .header(header::COOKIE, &auth_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email_enabled":false,"digest_hour_utc":15}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("save settings response");
    assert_eq!(save_settings.status(), StatusCode::FORBIDDEN);
}
