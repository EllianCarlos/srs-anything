use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait, Value};

use crate::{
    models::{
        EmailDeliveryLog, IngestProblemInput, IntegrationToken, NotificationPreference,
        ProblemCard, ProblemEvent, ProblemStatus, ReviewEvent, User, hash_token,
        make_event_dedup_key,
    },
    repositories::{
        error::RepoError,
        traits::{
            AuthRepository, EventRepository, IntegrationTokenRepository, NewIntegrationToken,
            ReviewRepository, SettingsRepository,
        },
    },
    srs::{Grade, SrsSchedule, next_interval_index},
};

#[derive(Clone)]
pub struct PostgresRepository {
    db: DatabaseConnection,
    schedule: SrsSchedule,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection, schedule: SrsSchedule) -> Self {
        Self { db, schedule }
    }

    fn status_to_db(status: ProblemStatus) -> String {
        match status {
            ProblemStatus::Solved => "solved".to_owned(),
            ProblemStatus::Unsolved => "unsolved".to_owned(),
        }
    }

    fn status_from_db(raw: String) -> Result<ProblemStatus, RepoError> {
        match raw.as_str() {
            "solved" => Ok(ProblemStatus::Solved),
            "unsolved" => Ok(ProblemStatus::Unsolved),
            _ => Err(RepoError::Message(format!("unknown problem status: {raw}"))),
        }
    }

    fn grade_to_db(grade: Grade) -> String {
        match grade {
            Grade::Again => "again".to_owned(),
            Grade::Hard => "hard".to_owned(),
            Grade::Good => "good".to_owned(),
            Grade::Easy => "easy".to_owned(),
        }
    }

    fn grade_from_db(raw: String) -> Result<Grade, RepoError> {
        match raw.as_str() {
            "again" => Ok(Grade::Again),
            "hard" => Ok(Grade::Hard),
            "good" => Ok(Grade::Good),
            "easy" => Ok(Grade::Easy),
            _ => Err(RepoError::Message(format!("unknown review grade: {raw}"))),
        }
    }

    fn duration_at_index(&self, index: usize) -> Duration {
        self.schedule.duration_for_index(index)
    }
}

#[async_trait]
impl AuthRepository for PostgresRepository {
    async fn get_or_create_user(&self, email: &str) -> Result<User, RepoError> {
        let inserted = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO users (email)
                VALUES ($1)
                ON CONFLICT (email) DO NOTHING
                RETURNING id, email, created_at
                "#,
                vec![Value::from(email.to_owned())],
            ))
            .await?;

        let user_row = match inserted {
            Some(row) => row,
            None => self
                .db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT id, email, created_at FROM users WHERE email = $1",
                    vec![Value::from(email.to_owned())],
                ))
                .await?
                .ok_or_else(|| RepoError::Message("user not found after upsert".to_owned()))?,
        };

        let user = User {
            id: user_row.try_get("", "id")?,
            email: user_row.try_get("", "email")?,
            created_at: user_row.try_get("", "created_at")?,
        };

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO notification_preferences (user_id, email_enabled, digest_hour_utc)
                VALUES ($1, true, 12)
                ON CONFLICT (user_id) DO NOTHING
                "#,
                vec![Value::from(user.id)],
            ))
            .await?;

        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id, email, created_at FROM users WHERE email = $1",
                vec![Value::from(email.to_owned())],
            ))
            .await?;

        row.map(|user_row| {
            Ok(User {
                id: user_row.try_get("", "id")?,
                email: user_row.try_get("", "email")?,
                created_at: user_row.try_get("", "created_at")?,
            })
        })
        .transpose()
    }

    async fn create_magic_link(&self, user_id: i64) -> Result<String, RepoError> {
        let token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();

        let token_hash = hash_token(&token);
        let expires_at = Utc::now() + Duration::minutes(15);

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO magic_link_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
                vec![
                    Value::from(user_id),
                    Value::from(token_hash),
                    Value::from(expires_at),
                ],
            ))
            .await?;

        Ok(token)
    }

    async fn verify_magic_link(&self, token: &str) -> Result<Option<User>, RepoError> {
        let token_hash = hash_token(token);
        let now = Utc::now();
        let tx = self.db.begin().await?;

        let token_row = tx
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, user_id
                FROM magic_link_tokens
                WHERE token_hash = $1
                  AND expires_at > $2
                  AND consumed_at IS NULL
                ORDER BY id DESC
                LIMIT 1
                "#,
                vec![Value::from(token_hash), Value::from(now)],
            ))
            .await?;

        let Some(token_row) = token_row else {
            tx.rollback().await?;
            return Ok(None);
        };
        let token_id: i64 = token_row.try_get("", "id")?;
        let user_id: i64 = token_row.try_get("", "user_id")?;

        tx.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE magic_link_tokens SET consumed_at = $1 WHERE id = $2",
            vec![Value::from(now), Value::from(token_id)],
        ))
        .await?;

        let user_row = tx
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id, email, created_at FROM users WHERE id = $1",
                vec![Value::from(user_id)],
            ))
            .await?
            .ok_or_else(|| RepoError::Message("user not found while verifying link".to_owned()))?;

        let user = User {
            id: user_row.try_get("", "id")?,
            email: user_row.try_get("", "email")?,
            created_at: user_row.try_get("", "created_at")?,
        };

        tx.commit().await?;
        Ok(Some(user))
    }
}

#[async_trait]
impl IntegrationTokenRepository for PostgresRepository {
    async fn create_integration_token(
        &self,
        user_id: i64,
        token_hash: &str,
        new_token: NewIntegrationToken,
    ) -> Result<IntegrationToken, RepoError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO integration_tokens
                    (user_id, token_hash, label, scopes, expires_at)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, user_id, token_hash, label, scopes, created_at, expires_at, revoked_at, last_used_at
                "#,
                vec![
                    Value::from(user_id),
                    Value::from(token_hash.to_owned()),
                    Value::from(new_token.label),
                    Value::from(new_token.scopes),
                    Value::from(new_token.expires_at),
                ],
            ))
            .await?
            .ok_or_else(|| RepoError::Message("failed to insert integration token".to_owned()))?;

        Ok(IntegrationToken {
            id: row.try_get("", "id")?,
            user_id: row.try_get("", "user_id")?,
            token_hash: row.try_get("", "token_hash")?,
            label: row.try_get("", "label")?,
            scopes: row.try_get("", "scopes")?,
            created_at: row.try_get("", "created_at")?,
            expires_at: row.try_get("", "expires_at")?,
            revoked_at: row.try_get("", "revoked_at")?,
            last_used_at: row.try_get("", "last_used_at")?,
        })
    }

    async fn list_integration_tokens(
        &self,
        user_id: i64,
    ) -> Result<Vec<IntegrationToken>, RepoError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, user_id, token_hash, label, scopes, created_at, expires_at, revoked_at, last_used_at
                FROM integration_tokens
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                vec![Value::from(user_id)],
            ))
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(IntegrationToken {
                    id: row.try_get("", "id")?,
                    user_id: row.try_get("", "user_id")?,
                    token_hash: row.try_get("", "token_hash")?,
                    label: row.try_get("", "label")?,
                    scopes: row.try_get("", "scopes")?,
                    created_at: row.try_get("", "created_at")?,
                    expires_at: row.try_get("", "expires_at")?,
                    revoked_at: row.try_get("", "revoked_at")?,
                    last_used_at: row.try_get("", "last_used_at")?,
                })
            })
            .collect()
    }

    async fn revoke_integration_token(
        &self,
        user_id: i64,
        token_id: i64,
    ) -> Result<bool, RepoError> {
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                UPDATE integration_tokens
                SET revoked_at = NOW()
                WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
                "#,
                vec![Value::from(token_id), Value::from(user_id)],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn user_from_integration_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<User>, RepoError> {
        let now = Utc::now();
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT u.id, u.email, u.created_at
                FROM integration_tokens t
                INNER JOIN users u ON u.id = t.user_id
                WHERE t.token_hash = $1
                  AND t.revoked_at IS NULL
                  AND (t.expires_at IS NULL OR t.expires_at > $2)
                LIMIT 1
                "#,
                vec![Value::from(token_hash.to_owned()), Value::from(now)],
            ))
            .await?;

        row.map(|user_row| {
            Ok(User {
                id: user_row.try_get("", "id")?,
                email: user_row.try_get("", "email")?,
                created_at: user_row.try_get("", "created_at")?,
            })
        })
        .transpose()
    }

    async fn touch_integration_token_usage(&self, token_hash: &str) -> Result<(), RepoError> {
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE integration_tokens SET last_used_at = NOW() WHERE token_hash = $1",
                vec![Value::from(token_hash.to_owned())],
            ))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl EventRepository for PostgresRepository {
    async fn ingest_event(&self, payload: IngestProblemInput) -> Result<ProblemEvent, RepoError> {
        let status_for_dedup = payload.status.clone();
        let status_for_event = payload.status.clone();
        let dedup_key = make_event_dedup_key(
            payload.user_id,
            &payload.source,
            &payload.problem_slug,
            &status_for_dedup,
            payload.occurred_at,
        );

        let inserted = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO problem_events
                  (user_id, source, problem_slug, title, url, status, occurred_at, dedup_key)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (dedup_key) DO NOTHING
                RETURNING id, user_id, source, problem_slug, title, url, status, occurred_at, dedup_key
                "#,
                vec![
                    Value::from(payload.user_id),
                    Value::from(payload.source.clone()),
                    Value::from(payload.problem_slug.clone()),
                    Value::from(payload.title.clone()),
                    Value::from(payload.url.clone()),
                    Value::from(Self::status_to_db(status_for_event)),
                    Value::from(payload.occurred_at),
                    Value::from(dedup_key.clone()),
                ],
            ))
            .await?;

        let event_row = match inserted {
            Some(row) => row,
            None => self
                .db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    SELECT id, user_id, source, problem_slug, title, url, status, occurred_at, dedup_key
                    FROM problem_events
                    WHERE dedup_key = $1
                    "#,
                    vec![Value::from(dedup_key.clone())],
                ))
                .await?
                .ok_or_else(|| RepoError::Message("deduplicated event not found".to_owned()))?,
        };

        let interval_index: i32 = 0;
        let next_due_at = payload.occurred_at + self.duration_at_index(interval_index as usize);
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO problem_cards
                  (user_id, source, problem_slug, title, url, interval_index, next_due_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (user_id, source, problem_slug)
                DO UPDATE
                  SET title = EXCLUDED.title,
                      url = EXCLUDED.url,
                      interval_index = EXCLUDED.interval_index,
                      next_due_at = EXCLUDED.next_due_at
                "#,
                vec![
                    Value::from(payload.user_id),
                    Value::from(payload.source),
                    Value::from(payload.problem_slug),
                    Value::from(payload.title),
                    Value::from(payload.url),
                    Value::from(interval_index),
                    Value::from(next_due_at),
                ],
            ))
            .await?;

        Ok(ProblemEvent {
            id: event_row.try_get("", "id")?,
            user_id: event_row.try_get("", "user_id")?,
            source: event_row.try_get("", "source")?,
            problem_slug: event_row.try_get("", "problem_slug")?,
            title: event_row.try_get("", "title")?,
            url: event_row.try_get("", "url")?,
            status: Self::status_from_db(event_row.try_get::<String>("", "status")?)?,
            occurred_at: event_row.try_get("", "occurred_at")?,
            dedup_key: event_row.try_get("", "dedup_key")?,
        })
    }

    async fn latest_event_for_user(&self, user_id: i64) -> Result<Option<ProblemEvent>, RepoError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, user_id, source, problem_slug, title, url, status, occurred_at, dedup_key
                FROM problem_events
                WHERE user_id = $1
                ORDER BY occurred_at DESC
                LIMIT 1
                "#,
                vec![Value::from(user_id)],
            ))
            .await?;

        row.map(|event_row| {
            Ok(ProblemEvent {
                id: event_row.try_get("", "id")?,
                user_id: event_row.try_get("", "user_id")?,
                source: event_row.try_get("", "source")?,
                problem_slug: event_row.try_get("", "problem_slug")?,
                title: event_row.try_get("", "title")?,
                url: event_row.try_get("", "url")?,
                status: Self::status_from_db(event_row.try_get::<String>("", "status")?)?,
                occurred_at: event_row.try_get("", "occurred_at")?,
                dedup_key: event_row.try_get("", "dedup_key")?,
            })
        })
        .transpose()
    }
}

#[async_trait]
impl ReviewRepository for PostgresRepository {
    async fn due_cards(
        &self,
        user_id: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProblemCard>, RepoError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, user_id, source, problem_slug, title, url, interval_index, next_due_at
                FROM problem_cards
                WHERE user_id = $1 AND next_due_at <= $2
                ORDER BY next_due_at ASC
                "#,
                vec![Value::from(user_id), Value::from(now)],
            ))
            .await?;

        rows.into_iter()
            .map(|row| {
                let index: i32 = row.try_get("", "interval_index")?;
                Ok(ProblemCard {
                    id: row.try_get("", "id")?,
                    user_id: row.try_get("", "user_id")?,
                    source: row.try_get("", "source")?,
                    problem_slug: row.try_get("", "problem_slug")?,
                    title: row.try_get("", "title")?,
                    url: row.try_get("", "url")?,
                    interval_index: index.max(0) as usize,
                    next_due_at: row.try_get("", "next_due_at")?,
                })
            })
            .collect()
    }

    async fn upcoming_cards(&self, user_id: i64) -> Result<Vec<ProblemCard>, RepoError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, user_id, source, problem_slug, title, url, interval_index, next_due_at
                FROM problem_cards
                WHERE user_id = $1
                ORDER BY next_due_at ASC
                LIMIT 10
                "#,
                vec![Value::from(user_id)],
            ))
            .await?;

        rows.into_iter()
            .map(|row| {
                let index: i32 = row.try_get("", "interval_index")?;
                Ok(ProblemCard {
                    id: row.try_get("", "id")?,
                    user_id: row.try_get("", "user_id")?,
                    source: row.try_get("", "source")?,
                    problem_slug: row.try_get("", "problem_slug")?,
                    title: row.try_get("", "title")?,
                    url: row.try_get("", "url")?,
                    interval_index: index.max(0) as usize,
                    next_due_at: row.try_get("", "next_due_at")?,
                })
            })
            .collect()
    }

    async fn grade_card(
        &self,
        user_id: i64,
        card_id: i64,
        grade: Grade,
    ) -> Result<Option<ReviewEvent>, RepoError> {
        let tx = self.db.begin().await?;

        let card_row = tx
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, interval_index
                FROM problem_cards
                WHERE id = $1 AND user_id = $2
                FOR UPDATE
                "#,
                vec![Value::from(card_id), Value::from(user_id)],
            ))
            .await?;

        let Some(card_row) = card_row else {
            tx.rollback().await?;
            return Ok(None);
        };

        let current_index: i32 = card_row.try_get("", "interval_index")?;
        let next_index = next_interval_index(
            current_index.max(0) as usize,
            grade,
            self.schedule.max_index(),
        ) as i32;
        let reviewed_at = Utc::now();
        let next_due_at = reviewed_at + self.duration_at_index(next_index as usize);

        tx.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE problem_cards SET interval_index = $1, next_due_at = $2 WHERE id = $3",
            vec![
                Value::from(next_index),
                Value::from(next_due_at),
                Value::from(card_id),
            ],
        ))
        .await?;

        let review_row = tx
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO review_events (card_id, user_id, grade, reviewed_at, next_due_at)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, card_id, user_id, grade, reviewed_at, next_due_at
                "#,
                vec![
                    Value::from(card_id),
                    Value::from(user_id),
                    Value::from(Self::grade_to_db(grade)),
                    Value::from(reviewed_at),
                    Value::from(next_due_at),
                ],
            ))
            .await?
            .ok_or_else(|| RepoError::Message("failed to insert review".to_owned()))?;

        tx.commit().await?;

        Ok(Some(ReviewEvent {
            id: review_row.try_get("", "id")?,
            card_id: review_row.try_get("", "card_id")?,
            user_id: review_row.try_get("", "user_id")?,
            grade: Self::grade_from_db(review_row.try_get::<String>("", "grade")?)?,
            reviewed_at: review_row.try_get("", "reviewed_at")?,
            next_due_at: review_row.try_get("", "next_due_at")?,
        }))
    }

    async fn user_history(&self, user_id: i64) -> Result<Vec<ReviewEvent>, RepoError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT id, card_id, user_id, grade, reviewed_at, next_due_at
                FROM review_events
                WHERE user_id = $1
                ORDER BY reviewed_at DESC
                "#,
                vec![Value::from(user_id)],
            ))
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ReviewEvent {
                    id: row.try_get("", "id")?,
                    card_id: row.try_get("", "card_id")?,
                    user_id: row.try_get("", "user_id")?,
                    grade: Self::grade_from_db(row.try_get::<String>("", "grade")?)?,
                    reviewed_at: row.try_get("", "reviewed_at")?,
                    next_due_at: row.try_get("", "next_due_at")?,
                })
            })
            .collect()
    }
}

#[async_trait]
impl SettingsRepository for PostgresRepository {
    async fn get_notification_preference(
        &self,
        user_id: i64,
    ) -> Result<Option<NotificationPreference>, RepoError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT user_id, email_enabled, digest_hour_utc
                FROM notification_preferences
                WHERE user_id = $1
                "#,
                vec![Value::from(user_id)],
            ))
            .await?;

        row.map(|pref_row| {
            let digest_hour: i32 = pref_row.try_get("", "digest_hour_utc")?;
            Ok(NotificationPreference {
                user_id: pref_row.try_get("", "user_id")?,
                email_enabled: pref_row.try_get("", "email_enabled")?,
                digest_hour_utc: digest_hour.clamp(0, 23) as u8,
            })
        })
        .transpose()
    }

    async fn set_notification_preference(
        &self,
        user_id: i64,
        email_enabled: bool,
        digest_hour_utc: u8,
    ) -> Result<Option<NotificationPreference>, RepoError> {
        let clamped_hour = digest_hour_utc.min(23) as i32;
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                UPDATE notification_preferences
                SET email_enabled = $1, digest_hour_utc = $2
                WHERE user_id = $3
                RETURNING user_id, email_enabled, digest_hour_utc
                "#,
                vec![
                    Value::from(email_enabled),
                    Value::from(clamped_hour),
                    Value::from(user_id),
                ],
            ))
            .await?;

        row.map(|pref_row| {
            let digest_hour: i32 = pref_row.try_get("", "digest_hour_utc")?;
            Ok(NotificationPreference {
                user_id: pref_row.try_get("", "user_id")?,
                email_enabled: pref_row.try_get("", "email_enabled")?,
                digest_hour_utc: digest_hour.clamp(0, 23) as u8,
            })
        })
        .transpose()
    }

    async fn list_users(&self) -> Result<Vec<User>, RepoError> {
        let rows = self
            .db
            .query_all(Statement::from_string(
                DbBackend::Postgres,
                "SELECT id, email, created_at FROM users ORDER BY id ASC".to_owned(),
            ))
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(User {
                    id: row.try_get("", "id")?,
                    email: row.try_get("", "email")?,
                    created_at: row.try_get("", "created_at")?,
                })
            })
            .collect()
    }

    async fn log_email(
        &self,
        user_id: i64,
        subject: &str,
        body: &str,
    ) -> Result<EmailDeliveryLog, RepoError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO email_delivery_logs (user_id, subject, body)
                VALUES ($1, $2, $3)
                RETURNING id, user_id, sent_at, subject, body
                "#,
                vec![
                    Value::from(user_id),
                    Value::from(subject.to_owned()),
                    Value::from(body.to_owned()),
                ],
            ))
            .await?
            .ok_or_else(|| RepoError::Message("failed to insert email delivery log".to_owned()))?;

        Ok(EmailDeliveryLog {
            id: row.try_get("", "id")?,
            user_id: row.try_get("", "user_id")?,
            sent_at: row.try_get("", "sent_at")?,
            subject: row.try_get("", "subject")?,
            body: row.try_get("", "body")?,
        })
    }
}
