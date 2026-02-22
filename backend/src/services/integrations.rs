use std::sync::Arc;

use chrono::{Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use tracing::{info, warn};

use crate::{
    dto::integrations::{
        CreateIntegrationTokenResponse, IntegrationTokenSummary, IntegrationsResponse,
    },
    errors::AppError,
    models::{User, hash_token},
    repositories::traits::{IntegrationTokenRepository, NewIntegrationToken},
};

use super::event::EventService;

#[derive(Clone)]
pub struct IntegrationsService {
    event_service: EventService,
    repo: Arc<dyn IntegrationTokenRepository>,
}

impl IntegrationsService {
    pub fn new(event_service: EventService, repo: Arc<dyn IntegrationTokenRepository>) -> Self {
        Self {
            event_service,
            repo,
        }
    }

    pub async fn get(&self, user_id: i64) -> Result<IntegrationsResponse, AppError> {
        let tokens = self
            .repo
            .list_integration_tokens(user_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let response = IntegrationsResponse {
            api_token_setup: vec![
                "Create an API token from this page (backend-generated, shown once).".to_owned(),
                "Use Send to Tampermonkey to store it in userscript storage.".to_owned(),
                "The script sends X-API-Key for ingestion requests.".to_owned(),
            ],
            latest_event: self.event_service.latest_for_user(user_id).await?,
            checklist: vec![
                "Install Tampermonkey extension".to_owned(),
                "Install script from integrations page".to_owned(),
                "Login with your LeetCode account".to_owned(),
                "Open or solve a problem and verify the event appears here".to_owned(),
            ],
            tokens: tokens
                .into_iter()
                .map(|token| IntegrationTokenSummary {
                    id: token.id,
                    label: token.label,
                    scopes: token.scopes,
                    created_at: token.created_at,
                    expires_at: token.expires_at,
                    revoked_at: token.revoked_at,
                    last_used_at: token.last_used_at,
                })
                .collect(),
        };
        info!(
            user_id,
            has_latest_event = response.latest_event.is_some(),
            token_count = response.tokens.len(),
            "integrations_built"
        );
        Ok(response)
    }

    pub async fn create_token(
        &self,
        user_id: i64,
        label: &str,
        expires_in_days: Option<i64>,
    ) -> Result<CreateIntegrationTokenResponse, AppError> {
        if label.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "token label cannot be empty".to_owned(),
            ));
        }
        let raw_secret: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        let token = format!("srs_it_{raw_secret}");
        let token_hash = hash_token(&token);
        let expires_at = expires_in_days
            .filter(|days| *days > 0)
            .map(|days| Utc::now() + Duration::days(days));
        let created = self
            .repo
            .create_integration_token(
                user_id,
                &token_hash,
                NewIntegrationToken {
                    label: label.trim().to_owned(),
                    scopes: vec!["events:write".to_owned()],
                    expires_at,
                },
            )
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let token_summary = IntegrationTokenSummary {
            id: created.id,
            label: created.label,
            scopes: created.scopes,
            created_at: created.created_at,
            expires_at: created.expires_at,
            revoked_at: created.revoked_at,
            last_used_at: created.last_used_at,
        };
        info!(
            user_id,
            token_id = token_summary.id,
            "integration_token_created"
        );
        Ok(CreateIntegrationTokenResponse {
            token,
            token_summary,
        })
    }

    pub async fn revoke_token(&self, user_id: i64, token_id: i64) -> Result<(), AppError> {
        let revoked = self
            .repo
            .revoke_integration_token(user_id, token_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        if !revoked {
            warn!(user_id, token_id, "integration_token_not_found");
            return Err(AppError::IntegrationTokenNotFound);
        }
        info!(user_id, token_id, "integration_token_revoked");
        Ok(())
    }

    pub async fn user_from_api_key(&self, token: Option<&str>) -> Result<User, AppError> {
        let raw = token.ok_or(AppError::MissingApiToken)?;
        let hash = hash_token(raw.trim());
        let user = self
            .repo
            .user_from_integration_token(&hash)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?
            .ok_or(AppError::InvalidApiToken)?;
        self.repo
            .touch_integration_token_usage(&hash)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::dto::integrations::IntegrationsResponse;

    #[test]
    fn integrations_response_uses_setup_steps_not_hint_token() {
        let response = IntegrationsResponse {
            api_token_setup: vec!["Create API token".to_owned()],
            latest_event: None,
            checklist: vec!["Install Tampermonkey extension".to_owned()],
            tokens: vec![],
        };

        let payload = serde_json::to_value(&response).expect("serialize integrations response");
        let object = payload
            .as_object()
            .expect("integrations response should serialize to an object");

        assert!(object.contains_key("api_token_setup"));
        assert!(object.contains_key("tokens"));
        assert_eq!(object.get("latest_event"), Some(&Value::Null));
    }
}
