use std::sync::Arc;

use chrono::{DateTime, Timelike, Utc};
use tracing::{info, warn};

use crate::{errors::AppError, ports::notification_sender::NotificationSender};

use super::{review::ReviewService, settings::SettingsService};

#[derive(Clone)]
pub struct NotificationService {
    settings_service: SettingsService,
    review_service: ReviewService,
    sender: Arc<dyn NotificationSender>,
}

impl NotificationService {
    pub fn new(
        settings_service: SettingsService,
        review_service: ReviewService,
        sender: Arc<dyn NotificationSender>,
    ) -> Self {
        Self {
            settings_service,
            review_service,
            sender,
        }
    }

    pub async fn process_digests_once(&self, now: DateTime<Utc>) -> Result<(), AppError> {
        info!(tick_hour_utc = now.hour(), "digest_tick_started");
        let users = self.settings_service.list_users().await?;
        info!(users_count = users.len(), "digest_users_loaded");
        for user in users {
            let pref = match self.settings_service.get(user.id).await {
                Ok(pref) => pref,
                Err(AppError::SettingsNotFound) => {
                    warn!(user_id = user.id, "digest_settings_not_found");
                    continue;
                }
                Err(err) => return Err(err),
            };
            if !pref.email_enabled || pref.digest_hour_utc != now.hour() as u8 {
                info!(
                    user_id = user.id,
                    email_enabled = pref.email_enabled,
                    digest_hour_utc = pref.digest_hour_utc,
                    "digest_skipped_by_preference"
                );
                continue;
            }
            let due = self.review_service.due_cards(user.id, now).await?;
            if due.is_empty() {
                info!(user_id = user.id, "digest_skipped_no_due_cards");
                continue;
            }
            let subject = format!("SRS reminder: {} reviews due", due.len());
            let body = format!(
                "You have {} due reviews. Open your dashboard to continue.",
                due.len()
            );
            self.sender
                .send_digest(user.id, &subject, &body)
                .await
                .map_err(|err| AppError::Internal(err.to_string()))?;
            info!(user_id = user.id, due_count = due.len(), "digest_queued");
        }
        info!("digest_tick_finished");
        Ok(())
    }
}
