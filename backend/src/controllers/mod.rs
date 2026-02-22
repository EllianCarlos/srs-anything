pub mod auth;
pub mod dashboard;
pub mod events;
pub mod integrations;
pub mod reviews;
pub mod settings;

use axum::http::HeaderMap;

pub fn auth_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| {
            raw.split(';').find_map(|chunk| {
                let mut parts = chunk.trim().splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some("srs_auth"), Some(value)) if !value.is_empty() => Some(value),
                    _ => None,
                }
            })
        })
}

pub fn api_key(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
}
