use std::collections::HashSet;

use crate::services::{
    auth::AuthService, dashboard::DashboardService, event::EventService,
    integrations::IntegrationsService, notification::NotificationService, review::ReviewService,
    settings::SettingsService,
};

#[derive(Clone)]
pub struct SecurityConfig {
    pub cookie_secure: bool,
    pub allowed_origins: HashSet<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub event_service: EventService,
    pub review_service: ReviewService,
    pub dashboard_service: DashboardService,
    pub settings_service: SettingsService,
    pub integrations_service: IntegrationsService,
    pub notification_service: NotificationService,
    pub security: SecurityConfig,
}
