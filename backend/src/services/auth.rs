use std::{env, sync::Arc};

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{errors::AppError, models::User, repositories::traits::AuthRepository};

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub jwt_expiration_secs: i64,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let srs_profile = env::var("SRS_PROFILE").unwrap_or_default();
        let app_env = env::var("APP_ENV").unwrap_or_default();
        let require_jwt_secret = srs_profile == "prod" || app_env == "production";

        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(value) if !value.trim().is_empty() => value,
            _ if require_jwt_secret => {
                panic!("JWT_SECRET must be set when SRS_PROFILE=prod or APP_ENV=production")
            }
            _ => {
                warn!("JWT_SECRET missing, using insecure development fallback");
                "dev-insecure-secret-change-me".to_owned()
            }
        };

        Self {
            jwt_secret,
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "srs-anything".to_owned()),
            jwt_audience: env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "srs-anything-web".to_owned()),
            jwt_expiration_secs: env::var("JWT_EXPIRATION_SECS")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(60 * 60 * 24 * 30),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>, config: AuthConfig) -> Self {
        let secret_bytes = config.jwt_secret.as_bytes();
        Self {
            repo,
            encoding_key: EncodingKey::from_secret(secret_bytes),
            decoding_key: DecodingKey::from_secret(secret_bytes),
            config,
        }
    }

    pub async fn request_magic_link(&self, email: &str) -> Result<String, AppError> {
        if !email.contains('@') {
            warn!("auth_invalid_email");
            return Err(AppError::InvalidEmail);
        }
        let user = self
            .repo
            .get_or_create_user(&email.to_lowercase())
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let token = self
            .repo
            .create_magic_link(user.id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        info!(user_id = user.id, "auth_magic_link_created");
        Ok(token)
    }

    pub async fn verify_magic_link(&self, token: &str) -> Result<(User, String), AppError> {
        let user = self
            .repo
            .verify_magic_link(token)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?
            .ok_or_else(|| {
                warn!("auth_magic_link_invalid_or_expired");
                AppError::InvalidOrExpiredMagicLink
            })?;
        let jwt = self.issue_jwt(&user.email)?;
        info!(user_id = user.id, "auth_magic_link_verified");
        Ok((user, jwt))
    }

    pub async fn user_from_jwt_cookie(&self, token: Option<&str>) -> Result<User, AppError> {
        let token = token.ok_or(AppError::MissingAuthCookie)?;
        let claims = self.decode_jwt(token)?;
        let user = self
            .repo
            .get_user_by_email(&claims.sub)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?
            .ok_or_else(|| {
                warn!("auth_invalid_session");
                AppError::InvalidSession
            })?;
        info!(user_id = user.id, "auth_jwt_resolved");
        Ok(user)
    }

    pub fn session_max_age_secs(&self) -> i64 {
        self.config.jwt_expiration_secs
    }

    fn issue_jwt(&self, email: &str) -> Result<String, AppError> {
        let iat = Utc::now().timestamp();
        let exp = (Utc::now() + Duration::seconds(self.config.jwt_expiration_secs)).timestamp();
        let claims = Claims {
            sub: email.to_owned(),
            iss: self.config.jwt_issuer.clone(),
            aud: self.config.jwt_audience.clone(),
            iat: iat as usize,
            exp: exp as usize,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|err| AppError::Internal(format!("failed to issue jwt: {err}")))
    }

    fn decode_jwt(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(std::slice::from_ref(&self.config.jwt_issuer));
        validation.set_audience(std::slice::from_ref(&self.config.jwt_audience));
        validation.set_required_spec_claims(&["exp", "iat", "iss", "aud", "sub"]);
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|_| AppError::InvalidSession)
    }
}
