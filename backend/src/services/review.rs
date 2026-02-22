use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::{
    errors::AppError,
    models::{ProblemCard, ReviewEvent},
    repositories::traits::ReviewRepository,
    srs::Grade,
};

#[derive(Clone)]
pub struct ReviewService {
    repo: Arc<dyn ReviewRepository>,
}

impl ReviewService {
    pub fn new(repo: Arc<dyn ReviewRepository>) -> Self {
        Self { repo }
    }

    pub async fn due_cards(
        &self,
        user_id: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProblemCard>, AppError> {
        let cards = self
            .repo
            .due_cards(user_id, now)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        info!(user_id, due_count = cards.len(), "review_due_cards");
        Ok(cards)
    }

    pub async fn upcoming_cards(&self, user_id: i64) -> Result<Vec<ProblemCard>, AppError> {
        let cards = self
            .repo
            .upcoming_cards(user_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        info!(
            user_id,
            upcoming_count = cards.len(),
            "review_upcoming_cards"
        );
        Ok(cards)
    }

    pub async fn grade_card(
        &self,
        user_id: i64,
        card_id: i64,
        grade: Grade,
    ) -> Result<ReviewEvent, AppError> {
        let maybe_review = self
            .repo
            .grade_card(user_id, card_id, grade)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;

        let review = maybe_review.ok_or_else(|| {
            warn!(user_id, card_id, "review_grade_card_not_found");
            AppError::CardNotFound
        })?;
        info!(user_id, card_id, review_id = review.id, "review_graded");
        Ok(review)
    }

    pub async fn history(&self, user_id: i64) -> Result<Vec<ReviewEvent>, AppError> {
        let history = self
            .repo
            .user_history(user_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        info!(user_id, history_count = history.len(), "review_history");
        Ok(history)
    }
}
