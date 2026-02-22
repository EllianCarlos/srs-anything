use std::sync::Arc;

use async_trait::async_trait;

use crate::repositories::{error::RepoError, traits::SettingsRepository};

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_digest(&self, user_id: i64, subject: &str, body: &str) -> Result<(), RepoError>;
}

#[derive(Clone)]
pub struct StoreNotificationSender {
    settings_repo: Arc<dyn SettingsRepository>,
}

impl StoreNotificationSender {
    pub fn new(settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { settings_repo }
    }
}

#[async_trait]
impl NotificationSender for StoreNotificationSender {
    async fn send_digest(&self, user_id: i64, subject: &str, body: &str) -> Result<(), RepoError> {
        self.settings_repo.log_email(user_id, subject, body).await?;
        Ok(())
    }
}
