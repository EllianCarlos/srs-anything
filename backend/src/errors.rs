use axum::{Json, http::StatusCode};
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
}

impl ApiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing bearer token")]
    MissingBearerToken,
    #[error("Missing auth cookie")]
    MissingAuthCookie,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Missing API token")]
    MissingApiToken,
    #[error("Invalid API token")]
    InvalidApiToken,
    #[error("Integration token not found")]
    IntegrationTokenNotFound,
    #[error("Invalid email")]
    InvalidEmail,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid or expired magic link")]
    InvalidOrExpiredMagicLink,
    #[error("Card not found")]
    CardNotFound,
    #[error("Settings not found")]
    SettingsNotFound,
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn to_http(&self) -> (StatusCode, Json<ApiError>) {
        let status = match self {
            Self::MissingBearerToken
            | Self::MissingAuthCookie
            | Self::InvalidSession
            | Self::InvalidOrExpiredMagicLink
            | Self::MissingApiToken
            | Self::InvalidApiToken => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidEmail | Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::CardNotFound | Self::SettingsNotFound | Self::IntegrationTokenNotFound => {
                StatusCode::NOT_FOUND
            }
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            error!(error_kind = %self, http_status = status.as_u16(), "request_failed");
        } else {
            warn!(error_kind = %self, http_status = status.as_u16(), "request_rejected");
        }
        (status, Json(ApiError::new(self.to_string())))
    }
}
