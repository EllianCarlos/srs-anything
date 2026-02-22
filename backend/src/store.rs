use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distr::Alphanumeric};

use crate::models::{
    EmailDeliveryLog, IngestProblemInput, MagicLinkToken, NotificationPreference, ProblemCard,
    ProblemEvent, ReviewEvent, Session, User, hash_token, make_event_dedup_key,
};
use crate::srs::{Grade, SrsSchedule, next_interval_index};

#[derive(Debug, Default)]
pub struct InMemoryStore {
    pub users: HashMap<i64, User>,
    pub users_by_email: HashMap<String, i64>,
    pub magic_tokens: HashMap<i64, MagicLinkToken>,
    pub sessions: HashMap<i64, Session>,
    pub events: HashMap<i64, ProblemEvent>,
    pub cards: HashMap<i64, ProblemCard>,
    pub card_index: HashMap<String, i64>,
    pub reviews: HashMap<i64, ReviewEvent>,
    pub notification_preferences: HashMap<i64, NotificationPreference>,
    pub email_logs: HashMap<i64, EmailDeliveryLog>,
    pub schedule: SrsSchedule,
    dedup: HashSet<String>,
    next_id: i64,
}

impl InMemoryStore {
    pub fn new_with_schedule(schedule: SrsSchedule) -> Self {
        Self {
            schedule,
            next_id: 1,
            ..Self::default()
        }
    }

    fn new_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get_or_create_user(&mut self, email: &str) -> User {
        if let Some(user_id) = self.users_by_email.get(email) {
            return self.users.get(user_id).expect("user must exist").clone();
        }
        let user = User {
            id: self.new_id(),
            email: email.to_owned(),
            created_at: Utc::now(),
        };
        self.users_by_email.insert(email.to_owned(), user.id);
        self.notification_preferences.insert(
            user.id,
            NotificationPreference {
                user_id: user.id,
                email_enabled: true,
                digest_hour_utc: 12,
            },
        );
        self.users.insert(user.id, user.clone());
        user
    }

    pub fn create_magic_link(&mut self, user_id: i64) -> String {
        let token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        let record = MagicLinkToken {
            id: self.new_id(),
            user_id,
            token_hash: hash_token(&token),
            expires_at: Utc::now() + Duration::minutes(15),
            consumed_at: None,
        };
        self.magic_tokens.insert(record.id, record);
        token
    }

    pub fn verify_magic_link(&mut self, token: &str) -> Option<(User, String)> {
        let token_hash = hash_token(token);
        let now = Utc::now();
        let token_id = self.magic_tokens.iter().find_map(|(id, rec)| {
            (rec.token_hash == token_hash && rec.expires_at > now && rec.consumed_at.is_none())
                .then_some(*id)
        })?;
        let user_id = self.magic_tokens.get(&token_id)?.user_id;
        if let Some(rec) = self.magic_tokens.get_mut(&token_id) {
            rec.consumed_at = Some(now);
        }
        let session_token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        let session = Session {
            id: self.new_id(),
            user_id,
            session_token_hash: hash_token(&session_token),
            expires_at: now + Duration::days(30),
        };
        self.sessions.insert(session.id, session);
        self.users
            .get(&user_id)
            .cloned()
            .map(|user| (user, session_token))
    }

    pub fn user_from_session(&self, bearer_token: &str) -> Option<User> {
        let now = Utc::now();
        let token_hash = hash_token(bearer_token);
        let user_id = self.sessions.values().find_map(|session| {
            (session.session_token_hash == token_hash && session.expires_at > now)
                .then_some(session.user_id)
        })?;
        self.users.get(&user_id).cloned()
    }

    pub fn revoke_session(&mut self, bearer_token: &str) {
        let token_hash = hash_token(bearer_token);
        let to_remove = self
            .sessions
            .iter()
            .find_map(|(id, session)| (session.session_token_hash == token_hash).then_some(*id));
        if let Some(id) = to_remove {
            self.sessions.remove(&id);
        }
    }

    pub fn ingest_event(&mut self, payload: IngestProblemInput) -> ProblemEvent {
        let dedup_key = make_event_dedup_key(
            payload.user_id,
            &payload.source,
            &payload.problem_slug,
            &payload.status,
            payload.occurred_at,
        );
        if self.dedup.contains(&dedup_key) {
            return self
                .events
                .values()
                .find(|event| event.dedup_key == dedup_key)
                .expect("dedup event must exist")
                .clone();
        }
        self.dedup.insert(dedup_key.clone());
        let event = ProblemEvent {
            id: self.new_id(),
            user_id: payload.user_id,
            source: payload.source.clone(),
            problem_slug: payload.problem_slug.clone(),
            title: payload.title.clone(),
            url: payload.url.clone(),
            status: payload.status.clone(),
            occurred_at: payload.occurred_at,
            dedup_key,
        };
        self.events.insert(event.id, event.clone());

        let card_key = format!(
            "{}:{}:{}",
            payload.user_id, payload.source, payload.problem_slug
        );
        let card_id = if let Some(id) = self.card_index.get(&card_key) {
            *id
        } else {
            let id = self.new_id();
            self.card_index.insert(card_key, id);
            id
        };

        let interval_index = 0;
        let next_due_at = payload.occurred_at + self.schedule.duration_for_index(interval_index);
        let card = ProblemCard {
            id: card_id,
            user_id: payload.user_id,
            source: payload.source,
            problem_slug: payload.problem_slug,
            title: payload.title,
            url: payload.url,
            interval_index,
            next_due_at,
        };
        self.cards.insert(card_id, card);
        event
    }

    pub fn due_cards(&self, user_id: i64, now: DateTime<Utc>) -> Vec<ProblemCard> {
        let mut cards: Vec<_> = self
            .cards
            .values()
            .filter(|card| card.user_id == user_id && card.next_due_at <= now)
            .cloned()
            .collect();
        cards.sort_by_key(|card| card.next_due_at);
        cards
    }

    pub fn upcoming_cards(&self, user_id: i64) -> Vec<ProblemCard> {
        let mut cards: Vec<_> = self
            .cards
            .values()
            .filter(|card| card.user_id == user_id)
            .cloned()
            .collect();
        cards.sort_by_key(|card| card.next_due_at);
        cards.truncate(10);
        cards
    }

    pub fn grade_card(&mut self, user_id: i64, card_id: i64, grade: Grade) -> Option<ReviewEvent> {
        let next_due_at = {
            let card = self.cards.get_mut(&card_id)?;
            if card.user_id != user_id {
                return None;
            }
            card.interval_index =
                next_interval_index(card.interval_index, grade, self.schedule.max_index());
            card.next_due_at = Utc::now() + self.schedule.duration_for_index(card.interval_index);
            card.next_due_at
        };
        let review = ReviewEvent {
            id: self.new_id(),
            card_id,
            user_id,
            grade,
            reviewed_at: Utc::now(),
            next_due_at,
        };
        self.reviews.insert(review.id, review.clone());
        Some(review)
    }

    pub fn user_history(&self, user_id: i64) -> Vec<ReviewEvent> {
        let mut items: Vec<_> = self
            .reviews
            .values()
            .filter(|entry| entry.user_id == user_id)
            .cloned()
            .collect();
        items.sort_by_key(|entry| std::cmp::Reverse(entry.reviewed_at));
        items
    }

    pub fn get_notification_preference(&self, user_id: i64) -> Option<NotificationPreference> {
        self.notification_preferences.get(&user_id).cloned()
    }

    pub fn set_notification_preference(
        &mut self,
        user_id: i64,
        email_enabled: bool,
        digest_hour_utc: u8,
    ) -> Option<NotificationPreference> {
        let pref = self.notification_preferences.get_mut(&user_id)?;
        pref.email_enabled = email_enabled;
        pref.digest_hour_utc = digest_hour_utc.min(23);
        Some(pref.clone())
    }

    pub fn log_email(&mut self, user_id: i64, subject: &str, body: &str) -> EmailDeliveryLog {
        let log = EmailDeliveryLog {
            id: self.new_id(),
            user_id,
            sent_at: Utc::now(),
            subject: subject.to_owned(),
            body: body.to_owned(),
        };
        self.email_logs.insert(log.id, log.clone());
        log
    }

    pub fn latest_event_for_user(&self, user_id: i64) -> Option<ProblemEvent> {
        self.events
            .values()
            .filter(|ev| ev.user_id == user_id)
            .max_by_key(|ev| ev.occurred_at)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::models::{IngestProblemInput, ProblemStatus};
    use crate::srs::{IntervalUnit, ScheduleProfile, SrsSchedule};

    use super::InMemoryStore;

    #[test]
    fn deduplicates_ingestion() {
        let mut store = InMemoryStore::new_with_schedule(SrsSchedule::default());
        let user = store.get_or_create_user("a@b.com");
        let now = Utc::now();
        let first = store.ingest_event(IngestProblemInput {
            user_id: user.id,
            source: "leetcode".to_owned(),
            problem_slug: "two-sum".to_owned(),
            title: "Two Sum".to_owned(),
            url: "https://leetcode.com/problems/two-sum".to_owned(),
            status: ProblemStatus::Solved,
            occurred_at: now,
        });
        let second = store.ingest_event(IngestProblemInput {
            user_id: user.id,
            source: "leetcode".to_owned(),
            problem_slug: "two-sum".to_owned(),
            title: "Two Sum".to_owned(),
            url: "https://leetcode.com/problems/two-sum".to_owned(),
            status: ProblemStatus::Solved,
            occurred_at: now,
        });
        assert_eq!(first.id, second.id);
    }

    #[test]
    fn verifies_magic_link_once() {
        let mut store = InMemoryStore::new_with_schedule(SrsSchedule::default());
        let user = store.get_or_create_user("a@b.com");
        let token = store.create_magic_link(user.id);
        let ok = store.verify_magic_link(&token);
        assert!(ok.is_some());
        assert!(store.verify_magic_link(&token).is_none());
    }

    #[test]
    fn uses_injected_test_schedule_for_due_dates() {
        let schedule = SrsSchedule::from_profile(ScheduleProfile {
            unit: IntervalUnit::Minutes,
            intervals: vec![1, 3, 5],
        })
        .expect("valid test schedule");
        let mut store = InMemoryStore::new_with_schedule(schedule);
        let user = store.get_or_create_user("schedule@test.com");
        let now = Utc::now();

        store.ingest_event(IngestProblemInput {
            user_id: user.id,
            source: "leetcode".to_owned(),
            problem_slug: "binary-search".to_owned(),
            title: "Binary Search".to_owned(),
            url: "https://leetcode.com/problems/binary-search".to_owned(),
            status: ProblemStatus::Solved,
            occurred_at: now,
        });

        let card = store
            .cards
            .values()
            .find(|item| item.problem_slug == "binary-search")
            .expect("card created");
        assert!(card.next_due_at >= now + Duration::minutes(1));
        assert!(card.next_due_at < now + Duration::minutes(2));
    }
}
