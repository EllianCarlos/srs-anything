use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    errors::AppError, models::NotificationPreference, repositories::traits::SettingsRepository,
};

#[derive(Clone)]
pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, user_id: i64) -> Result<NotificationPreference, AppError> {
        let maybe_pref = self
            .repo
            .get_notification_preference(user_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;

        let pref = maybe_pref.ok_or_else(|| {
            warn!(user_id, "settings_not_found");
            AppError::SettingsNotFound
        })?;
        info!(
            user_id,
            email_enabled = pref.email_enabled,
            digest_hour_utc = pref.digest_hour_utc,
            "settings_loaded"
        );
        Ok(pref)
    }

    pub async fn save(
        &self,
        user_id: i64,
        email_enabled: bool,
        digest_hour_utc: u8,
    ) -> Result<NotificationPreference, AppError> {
        let maybe_pref = self
            .repo
            .set_notification_preference(user_id, email_enabled, digest_hour_utc)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;

        let pref = maybe_pref.ok_or_else(|| {
            warn!(user_id, "settings_not_found");
            AppError::SettingsNotFound
        })?;
        info!(
            user_id,
            email_enabled = pref.email_enabled,
            digest_hour_utc = pref.digest_hour_utc,
            "settings_saved"
        );
        Ok(pref)
    }

    pub async fn list_users(&self) -> Result<Vec<crate::models::User>, AppError> {
        self.repo
            .list_users()
            .await
            .map_err(|err| AppError::Internal(err.to_string()))
    }
}
