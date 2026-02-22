use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::info;

use crate::{
    app::state::AppState,
    dto::reviews::GradeRequest,
    extractors::authenticated_user::AuthenticatedUser,
    models::{ProblemCard, ReviewEvent},
};

pub async fn due_reviews(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Vec<ProblemCard>>, (StatusCode, Json<crate::errors::ApiError>)> {
    let cards = state
        .review_service
        .due_cards(user.id, chrono::Utc::now())
        .await
        .map_err(|err| err.to_http())?;
    info!(user_id = user.id, due_count = cards.len(), "reviews_due");
    Ok(Json(cards))
}

pub async fn grade_review(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(card_id): Path<i64>,
    Json(payload): Json<GradeRequest>,
) -> Result<Json<ReviewEvent>, (StatusCode, Json<crate::errors::ApiError>)> {
    info!(card_id, grade = ?payload.grade, "reviews_grade_request");
    let review = state
        .review_service
        .grade_card(user.id, card_id, payload.grade)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        card_id,
        review_id = review.id,
        "reviews_grade_created"
    );
    Ok(Json(review))
}

pub async fn history(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Vec<ReviewEvent>>, (StatusCode, Json<crate::errors::ApiError>)> {
    let history = state
        .review_service
        .history(user.id)
        .await
        .map_err(|err| err.to_http())?;
    info!(
        user_id = user.id,
        history_count = history.len(),
        "reviews_history"
    );
    Ok(Json(history))
}
