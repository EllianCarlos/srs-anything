use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Debug, Deserialize)]
pub struct RequestMagicLink {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct RequestMagicLinkResponse {
    pub sent: bool,
    pub dev_magic_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLink {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyMagicLinkResponse {
    pub user: User,
}
