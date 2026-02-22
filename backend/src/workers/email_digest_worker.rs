use crate::app::state::AppState;
use tracing::{Instrument, error, info};

pub async fn email_digest_worker(state: AppState) {
    info!(
        requestId = "worker-email-digest",
        "email_digest_worker_started"
    );
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let request_id = format!("worker-email-digest-{:032x}", rand::random::<u128>());
        let span = tracing::info_span!("email_digest_tick", requestId = %request_id);
        if let Err(error_value) = state
            .notification_service
            .process_digests_once(chrono::Utc::now())
            .instrument(span)
            .await
        {
            error!(error = %error_value, "email_digest_tick_failed");
        }
    }
}
