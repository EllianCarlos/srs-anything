use tracing::info;

use srs_anything_backend::{
    app::{bootstrap::build_state_from_provider, db::connect_from_env, routes::app_router},
    ports::schedule_provider::EnvScheduleProvider,
    workers::email_digest_worker::email_digest_worker,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .json()
        .flatten_event(true)
        .with_current_span(true)
        .init();

    let db = connect_from_env().await.expect("connect to postgres");
    let schedule_provider = EnvScheduleProvider;
    let state = build_state_from_provider(&schedule_provider, db);
    tokio::spawn(email_digest_worker(state.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind server");
    info!(
        requestId = "startup",
        listen_addr = "0.0.0.0:3000",
        "backend_started"
    );
    axum::serve(listener, app_router(state))
        .await
        .expect("serve app");
}
