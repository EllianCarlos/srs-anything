use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::ProblemEvent;

#[derive(Debug, Serialize)]
pub struct IntegrationsResponse {
    pub api_token_setup: Vec<String>,
    pub latest_event: Option<ProblemEvent>,
    pub checklist: Vec<String>,
    pub tokens: Vec<IntegrationTokenSummary>,
}

#[derive(Debug, Serialize)]
pub struct IntegrationTokenSummary {
    pub id: i64,
    pub label: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIntegrationTokenRequest {
    pub label: String,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateIntegrationTokenResponse {
    pub token: String,
    pub token_summary: IntegrationTokenSummary,
}
