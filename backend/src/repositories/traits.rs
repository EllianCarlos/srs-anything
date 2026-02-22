use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    models::{
        EmailDeliveryLog, IngestProblemInput, IntegrationToken, NotificationPreference,
        ProblemCard, ProblemEvent, ReviewEvent, User,
    },
    repositories::error::RepoError,
    srs::Grade,
};

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn get_or_create_user(&self, email: &str) -> Result<User, RepoError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn create_magic_link(&self, user_id: i64) -> Result<String, RepoError>;
    async fn verify_magic_link(&self, token: &str) -> Result<Option<User>, RepoError>;
}

#[derive(Debug, Clone)]
pub struct NewIntegrationToken {
    pub label: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait IntegrationTokenRepository: Send + Sync {
    async fn create_integration_token(
        &self,
        user_id: i64,
        token_hash: &str,
        new_token: NewIntegrationToken,
    ) -> Result<IntegrationToken, RepoError>;
    async fn list_integration_tokens(
        &self,
        user_id: i64,
    ) -> Result<Vec<IntegrationToken>, RepoError>;
    async fn revoke_integration_token(
        &self,
        user_id: i64,
        token_id: i64,
    ) -> Result<bool, RepoError>;
    async fn user_from_integration_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<User>, RepoError>;
    async fn touch_integration_token_usage(&self, token_hash: &str) -> Result<(), RepoError>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn ingest_event(&self, payload: IngestProblemInput) -> Result<ProblemEvent, RepoError>;
    async fn latest_event_for_user(&self, user_id: i64) -> Result<Option<ProblemEvent>, RepoError>;
}

#[async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn due_cards(
        &self,
        user_id: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProblemCard>, RepoError>;
    async fn upcoming_cards(&self, user_id: i64) -> Result<Vec<ProblemCard>, RepoError>;
    async fn grade_card(
        &self,
        user_id: i64,
        card_id: i64,
        grade: Grade,
    ) -> Result<Option<ReviewEvent>, RepoError>;
    async fn user_history(&self, user_id: i64) -> Result<Vec<ReviewEvent>, RepoError>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_notification_preference(
        &self,
        user_id: i64,
    ) -> Result<Option<NotificationPreference>, RepoError>;
    async fn set_notification_preference(
        &self,
        user_id: i64,
        email_enabled: bool,
        digest_hour_utc: u8,
    ) -> Result<Option<NotificationPreference>, RepoError>;
    async fn list_users(&self) -> Result<Vec<User>, RepoError>;
    async fn log_email(
        &self,
        user_id: i64,
        subject: &str,
        body: &str,
    ) -> Result<EmailDeliveryLog, RepoError>;
}
