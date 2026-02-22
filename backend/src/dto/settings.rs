use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SaveSettingsRequest {
    pub email_enabled: bool,
    pub digest_hour_utc: u8,
}
