use serde::Serialize;

use crate::models::ProblemEvent;

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub due_count: usize,
    pub upcoming_count: usize,
    pub leetcode_count: usize,
    pub neetcode_count: usize,
    pub latest_ingestion: Option<ProblemEvent>,
}
