use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use srs_anything_backend::app::db::connect_from_env;

pub async fn try_test_db() -> Option<DatabaseConnection> {
    match connect_from_env().await {
        Ok(db) => Some(db),
        Err(err) => {
            eprintln!("skipping postgres-backed integration test: {err}");
            None
        }
    }
}

pub async fn reset_db(db: &DatabaseConnection) {
    db.execute(Statement::from_string(
        DbBackend::Postgres,
        r#"
        TRUNCATE TABLE
          review_events,
          problem_cards,
          problem_events,
          integration_tokens,
          sessions,
          magic_link_tokens,
          email_delivery_logs,
          notification_preferences,
          users
        RESTART IDENTITY CASCADE
        "#
        .to_owned(),
    ))
    .await
    .expect("reset test database");
}
