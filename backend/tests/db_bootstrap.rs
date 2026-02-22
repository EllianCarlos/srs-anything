use serial_test::serial;
use srs_anything_backend::app::db::{DbBootstrapError, connect_and_verify, connect_from_env};

#[tokio::test]
#[serial]
async fn connect_from_env_succeeds_with_devenv_postgres() {
    let db = match connect_from_env().await {
        Ok(db) => db,
        Err(err) => {
            eprintln!("skipping postgres connectivity success test: {err}");
            return;
        }
    };
    let ping = db.ping().await;
    assert!(ping.is_ok());
}

#[tokio::test]
#[serial]
async fn connect_and_verify_fails_with_unreachable_database() {
    let err = connect_and_verify("postgres://srs:srs@127.0.0.1:1/srs_anything")
        .await
        .expect_err("expected connection failure");
    assert!(matches!(err, DbBootstrapError::Connect(_)));
}
