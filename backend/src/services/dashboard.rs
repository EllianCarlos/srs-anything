use crate::{dto::dashboard::DashboardResponse, errors::AppError};
use tracing::info;

use super::{event::EventService, review::ReviewService};

#[derive(Clone)]
pub struct DashboardService {
    review_service: ReviewService,
    event_service: EventService,
}

impl DashboardService {
    pub fn new(review_service: ReviewService, event_service: EventService) -> Self {
        Self {
            review_service,
            event_service,
        }
    }

    pub async fn dashboard_for_user(&self, user_id: i64) -> Result<DashboardResponse, AppError> {
        let due = self
            .review_service
            .due_cards(user_id, chrono::Utc::now())
            .await?;
        let upcoming = self.review_service.upcoming_cards(user_id).await?;
        let leetcode_count = upcoming
            .iter()
            .filter(|card| card.source == "leetcode")
            .count();
        let neetcode_count = upcoming
            .iter()
            .filter(|card| card.source == "neetcode")
            .count();
        let response = DashboardResponse {
            due_count: due.len(),
            upcoming_count: upcoming.len(),
            leetcode_count,
            neetcode_count,
            latest_ingestion: self.event_service.latest_for_user(user_id).await?,
        };
        info!(
            user_id,
            due_count = response.due_count,
            upcoming_count = response.upcoming_count,
            leetcode_count = response.leetcode_count,
            neetcode_count = response.neetcode_count,
            "dashboard_built"
        );
        Ok(response)
    }
}
