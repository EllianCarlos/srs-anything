use std::{collections::HashSet, env};

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::{
    app::state::AppState,
    ports::{
        notification_sender::{NotificationSender, StoreNotificationSender},
        schedule_provider::ScheduleProvider,
    },
    repositories::{
        postgres::PostgresRepository,
        traits::{
            AuthRepository, EventRepository, IntegrationTokenRepository, ReviewRepository,
            SettingsRepository,
        },
    },
    services::{
        auth::{AuthConfig, AuthService},
        dashboard::DashboardService,
        event::EventService,
        integrations::IntegrationsService,
        notification::NotificationService,
        review::ReviewService,
        settings::SettingsService,
    },
    srs::SrsSchedule,
};

pub fn build_state_with_schedule(schedule: SrsSchedule, db: DatabaseConnection) -> AppState {
    let repo = Arc::new(PostgresRepository::new(db, schedule));
    let auth_repo: Arc<dyn AuthRepository> = repo.clone();
    let event_repo: Arc<dyn EventRepository> = repo.clone();
    let review_repo: Arc<dyn ReviewRepository> = repo.clone();
    let settings_repo: Arc<dyn SettingsRepository> = repo.clone();
    let integration_repo: Arc<dyn IntegrationTokenRepository> = repo.clone();
    let notification_sender: Arc<dyn NotificationSender> =
        Arc::new(StoreNotificationSender::new(settings_repo.clone()));

    let auth_service = AuthService::new(auth_repo, AuthConfig::from_env());
    let event_service = EventService::new(event_repo);
    let review_service = ReviewService::new(review_repo);
    let dashboard_service = DashboardService::new(review_service.clone(), event_service.clone());
    let settings_service = SettingsService::new(settings_repo.clone());
    let integrations_service = IntegrationsService::new(event_service.clone(), integration_repo);
    let notification_service = NotificationService::new(
        settings_service.clone(),
        review_service.clone(),
        notification_sender,
    );
    let cookie_secure = env::var("COOKIE_SECURE")
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false);
    let allowed_origins = env::var("ALLOWED_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<HashSet<String>>()
        })
        .filter(|origins| !origins.is_empty())
        .unwrap_or_else(|| HashSet::from(["http://localhost:5173".to_owned()]));

    AppState {
        auth_service,
        event_service,
        review_service,
        dashboard_service,
        settings_service,
        integrations_service,
        notification_service,
        security: crate::app::state::SecurityConfig {
            cookie_secure,
            allowed_origins,
        },
    }
}

pub fn build_state_from_provider(
    provider: &dyn ScheduleProvider,
    db: DatabaseConnection,
) -> AppState {
    build_state_with_schedule(provider.load_schedule(), db)
}
