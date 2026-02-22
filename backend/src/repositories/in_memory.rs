use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::{
    models::{
        EmailDeliveryLog, IngestProblemInput, IntegrationToken, NotificationPreference,
        ProblemCard, ProblemEvent, ReviewEvent, User,
    },
    repositories::{
        error::RepoError,
        traits::{
            AuthRepository, EventRepository, IntegrationTokenRepository, NewIntegrationToken,
            ReviewRepository, SettingsRepository,
        },
    },
    srs::{Grade, SrsSchedule},
    store::InMemoryStore,
};

#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    inner: Arc<Mutex<InMemoryStore>>,
}

impl InMemoryRepository {
    pub fn new(schedule: SrsSchedule) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InMemoryStore::new_with_schedule(schedule))),
        }
    }
}

#[async_trait]
impl AuthRepository for InMemoryRepository {
    async fn get_or_create_user(&self, email: &str) -> Result<User, RepoError> {
        Ok(self.inner.lock().await.get_or_create_user(email))
    }

    async fn create_magic_link(&self, user_id: i64) -> Result<String, RepoError> {
        Ok(self.inner.lock().await.create_magic_link(user_id))
    }

    async fn verify_magic_link(&self, token: &str) -> Result<Option<User>, RepoError> {
        Ok(self.inner.lock().await.verify_magic_link(token))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        Ok(self.inner.lock().await.get_user_by_email(email))
    }
}

#[async_trait]
impl IntegrationTokenRepository for InMemoryRepository {
    async fn create_integration_token(
        &self,
        user_id: i64,
        token_hash: &str,
        new_token: NewIntegrationToken,
    ) -> Result<IntegrationToken, RepoError> {
        Ok(self.inner.lock().await.create_integration_token(
            user_id,
            token_hash.to_owned(),
            new_token.label,
            new_token.scopes,
            new_token.expires_at,
        ))
    }

    async fn list_integration_tokens(
        &self,
        user_id: i64,
    ) -> Result<Vec<IntegrationToken>, RepoError> {
        Ok(self.inner.lock().await.list_integration_tokens(user_id))
    }

    async fn revoke_integration_token(
        &self,
        user_id: i64,
        token_id: i64,
    ) -> Result<bool, RepoError> {
        Ok(self
            .inner
            .lock()
            .await
            .revoke_integration_token(user_id, token_id))
    }

    async fn user_from_integration_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<User>, RepoError> {
        Ok(self
            .inner
            .lock()
            .await
            .user_from_integration_token(token_hash))
    }

    async fn touch_integration_token_usage(&self, token_hash: &str) -> Result<(), RepoError> {
        self.inner
            .lock()
            .await
            .touch_integration_token_usage(token_hash);
        Ok(())
    }
}

#[async_trait]
impl EventRepository for InMemoryRepository {
    async fn ingest_event(&self, payload: IngestProblemInput) -> Result<ProblemEvent, RepoError> {
        Ok(self.inner.lock().await.ingest_event(payload))
    }

    async fn latest_event_for_user(&self, user_id: i64) -> Result<Option<ProblemEvent>, RepoError> {
        Ok(self.inner.lock().await.latest_event_for_user(user_id))
    }
}

#[async_trait]
impl ReviewRepository for InMemoryRepository {
    async fn due_cards(
        &self,
        user_id: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProblemCard>, RepoError> {
        Ok(self.inner.lock().await.due_cards(user_id, now))
    }

    async fn upcoming_cards(&self, user_id: i64) -> Result<Vec<ProblemCard>, RepoError> {
        Ok(self.inner.lock().await.upcoming_cards(user_id))
    }

    async fn grade_card(
        &self,
        user_id: i64,
        card_id: i64,
        grade: Grade,
    ) -> Result<Option<ReviewEvent>, RepoError> {
        Ok(self.inner.lock().await.grade_card(user_id, card_id, grade))
    }

    async fn user_history(&self, user_id: i64) -> Result<Vec<ReviewEvent>, RepoError> {
        Ok(self.inner.lock().await.user_history(user_id))
    }
}

#[async_trait]
impl SettingsRepository for InMemoryRepository {
    async fn get_notification_preference(
        &self,
        user_id: i64,
    ) -> Result<Option<NotificationPreference>, RepoError> {
        Ok(self.inner.lock().await.get_notification_preference(user_id))
    }

    async fn set_notification_preference(
        &self,
        user_id: i64,
        email_enabled: bool,
        digest_hour_utc: u8,
    ) -> Result<Option<NotificationPreference>, RepoError> {
        Ok(self.inner.lock().await.set_notification_preference(
            user_id,
            email_enabled,
            digest_hour_utc,
        ))
    }

    async fn list_users(&self) -> Result<Vec<User>, RepoError> {
        let users = self
            .inner
            .lock()
            .await
            .users
            .values()
            .cloned()
            .collect::<Vec<_>>();
        Ok(users)
    }

    async fn log_email(
        &self,
        user_id: i64,
        subject: &str,
        body: &str,
    ) -> Result<EmailDeliveryLog, RepoError> {
        Ok(self.inner.lock().await.log_email(user_id, subject, body))
    }
}
