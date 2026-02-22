use std::sync::Arc;

use serial_test::serial;
use srs_anything_backend::{
    models::{IngestProblemInput, ProblemStatus},
    repositories::{
        postgres::PostgresRepository,
        traits::{AuthRepository, EventRepository, ReviewRepository, SettingsRepository},
    },
    srs::SrsSchedule,
};

mod support;

async fn postgres_repos() -> Option<(AuthRepo, EventRepo, ReviewRepo, SettingsRepo)> {
    let db = support::db::try_test_db().await?;
    support::db::reset_db(&db).await;
    let repo = Arc::new(PostgresRepository::new(db, SrsSchedule::default()));
    Some((repo.clone(), repo.clone(), repo.clone(), repo))
}

type AuthRepo = Arc<dyn AuthRepository>;
type EventRepo = Arc<dyn EventRepository>;
type ReviewRepo = Arc<dyn ReviewRepository>;
type SettingsRepo = Arc<dyn SettingsRepository>;

#[tokio::test]
#[serial]
async fn auth_contract_supports_single_use_magic_links() {
    let Some((auth_repo, _, _, _)) = postgres_repos().await else {
        return;
    };
    let user = auth_repo
        .get_or_create_user("contracts@test.com")
        .await
        .expect("user");
    let token = auth_repo.create_magic_link(user.id).await.expect("token");
    let first = auth_repo.verify_magic_link(&token).await.expect("verify");
    assert!(first.is_some());
    let second = auth_repo
        .verify_magic_link(&token)
        .await
        .expect("verify second");
    assert!(second.is_none());
}

#[tokio::test]
#[serial]
async fn settings_contract_clamps_digest_hour() {
    let Some((auth_repo, _, _, settings_repo)) = postgres_repos().await else {
        return;
    };
    let user = auth_repo
        .get_or_create_user("settings@test.com")
        .await
        .expect("user");
    let updated = settings_repo
        .set_notification_preference(user.id, true, 250)
        .await
        .expect("set settings")
        .expect("settings");
    assert_eq!(updated.digest_hour_utc, 23);
}

#[tokio::test]
#[serial]
async fn event_and_review_contracts_round_trip() {
    let Some((auth_repo, event_repo, review_repo, _)) = postgres_repos().await else {
        return;
    };
    let user = auth_repo
        .get_or_create_user("flow@test.com")
        .await
        .expect("user");
    let now = chrono::Utc::now();

    let event = event_repo
        .ingest_event(IngestProblemInput {
            user_id: user.id,
            source: "leetcode".to_owned(),
            problem_slug: "two-sum".to_owned(),
            title: "Two Sum".to_owned(),
            url: "https://leetcode.com/problems/two-sum".to_owned(),
            status: ProblemStatus::Solved,
            occurred_at: now,
        })
        .await
        .expect("ingest");
    let dedup_event = event_repo
        .ingest_event(IngestProblemInput {
            user_id: user.id,
            source: "leetcode".to_owned(),
            problem_slug: "two-sum".to_owned(),
            title: "Two Sum".to_owned(),
            url: "https://leetcode.com/problems/two-sum".to_owned(),
            status: ProblemStatus::Solved,
            occurred_at: now,
        })
        .await
        .expect("ingest duplicate");
    assert_eq!(event.id, dedup_event.id);

    let upcoming = review_repo
        .upcoming_cards(user.id)
        .await
        .expect("upcoming cards");
    assert!(!upcoming.is_empty());
}
