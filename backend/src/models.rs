use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::srs::Grade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MagicLinkToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub session_token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub label: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProblemStatus {
    Solved,
    Unsolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemEvent {
    pub id: i64,
    pub user_id: i64,
    pub source: String,
    pub problem_slug: String,
    pub title: String,
    pub url: String,
    pub status: ProblemStatus,
    pub occurred_at: DateTime<Utc>,
    pub dedup_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemCard {
    pub id: i64,
    pub user_id: i64,
    pub source: String,
    pub problem_slug: String,
    pub title: String,
    pub url: String,
    pub interval_index: usize,
    pub next_due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEvent {
    pub id: i64,
    pub card_id: i64,
    pub user_id: i64,
    pub grade: Grade,
    pub reviewed_at: DateTime<Utc>,
    pub next_due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub user_id: i64,
    pub email_enabled: bool,
    pub digest_hour_utc: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDeliveryLog {
    pub id: i64,
    pub user_id: i64,
    pub sent_at: DateTime<Utc>,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct IngestProblemInput {
    pub user_id: i64,
    pub source: String,
    pub problem_slug: String,
    pub title: String,
    pub url: String,
    pub status: ProblemStatus,
    pub occurred_at: DateTime<Utc>,
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn make_event_dedup_key(
    user_id: i64,
    source: &str,
    slug: &str,
    status: &ProblemStatus,
    occurred_at: DateTime<Utc>,
) -> String {
    let bucket = occurred_at.format("%Y-%m-%dT%H").to_string();
    let status_str = match status {
        ProblemStatus::Solved => "solved",
        ProblemStatus::Unsolved => "unsolved",
    };
    format!("{user_id}:{source}:{slug}:{status_str}:{bucket}")
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{ProblemStatus, hash_token, make_event_dedup_key};

    #[test]
    fn hashes_stably() {
        assert_eq!(hash_token("abc"), hash_token("abc"));
        assert_ne!(hash_token("abc"), hash_token("abcd"));
    }

    #[test]
    fn dedup_key_has_bucket() {
        let now = Utc::now();
        let key = make_event_dedup_key(3, "leetcode", "two-sum", &ProblemStatus::Solved, now);
        assert!(key.contains("leetcode"));
        assert!(key.contains("two-sum"));
    }
}
