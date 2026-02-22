use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::models::ProblemStatus;

#[derive(Debug, Deserialize)]
pub struct IngestProblemEventRequest {
    pub source: String,
    pub problem_slug: String,
    pub title: String,
    pub url: String,
    pub status: ProblemStatus,
    pub occurred_at: DateTime<Utc>,
}
