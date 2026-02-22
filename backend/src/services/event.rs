use std::sync::Arc;
use tracing::info;

use crate::{
    errors::AppError,
    models::{IngestProblemInput, ProblemEvent},
    repositories::traits::EventRepository,
};

#[derive(Clone)]
pub struct EventService {
    repo: Arc<dyn EventRepository>,
}

impl EventService {
    pub fn new(repo: Arc<dyn EventRepository>) -> Self {
        Self { repo }
    }

    pub async fn ingest(&self, payload: IngestProblemInput) -> Result<ProblemEvent, AppError> {
        let user_id = payload.user_id;
        let source = payload.source.clone();
        let problem_slug = payload.problem_slug.clone();
        let event = self
            .repo
            .ingest_event(payload)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))?;
        info!(
            user_id,
            event_id = event.id,
            source = %source,
            problem_slug = %problem_slug,
            "event_ingested"
        );
        Ok(event)
    }

    pub async fn latest_for_user(&self, user_id: i64) -> Result<Option<ProblemEvent>, AppError> {
        self.repo
            .latest_event_for_user(user_id)
            .await
            .map_err(|err| AppError::Internal(err.to_string()))
    }
}
