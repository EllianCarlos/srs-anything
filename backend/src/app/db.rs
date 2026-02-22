use std::{env, fs, path::Path};

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use thiserror::Error;

const DEFAULT_SCHEMA_PATH: &str = "migrations/0001_init.sql";

#[derive(Debug, Error)]
pub enum DbBootstrapError {
    #[error("DATABASE_URL must be set: {0}")]
    MissingDatabaseUrl(#[from] env::VarError),
    #[error("Unable to connect to database: {0}")]
    Connect(#[from] sea_orm::DbErr),
    #[error("Unable to read schema file {path}: {source}")]
    ReadSchema {
        path: String,
        source: std::io::Error,
    },
}

pub async fn connect_from_env() -> Result<DatabaseConnection, DbBootstrapError> {
    let database_url = env::var("DATABASE_URL")?;
    connect_and_verify(&database_url).await
}

pub async fn connect_and_verify(
    database_url: &str,
) -> Result<DatabaseConnection, DbBootstrapError> {
    let db = Database::connect(database_url).await?;
    ensure_schema(&db, DEFAULT_SCHEMA_PATH).await?;
    verify_connection(&db).await?;
    Ok(db)
}

pub async fn ensure_schema(
    db: &DatabaseConnection,
    schema_path: impl AsRef<Path>,
) -> Result<(), DbBootstrapError> {
    let schema_path = schema_path.as_ref();
    let schema_sql =
        fs::read_to_string(schema_path).map_err(|source| DbBootstrapError::ReadSchema {
            path: schema_path.display().to_string(),
            source,
        })?;
    let schema_sql = schema_sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    for raw_statement in schema_sql.split(';') {
        let statement = raw_statement.trim();
        if statement.is_empty() {
            continue;
        }
        db.execute(Statement::from_string(
            DbBackend::Postgres,
            format!("{statement};"),
        ))
        .await?;
    }
    Ok(())
}

pub async fn verify_connection(db: &DatabaseConnection) -> Result<(), DbBootstrapError> {
    db.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT 1".to_owned(),
    ))
    .await?
    .ok_or_else(|| sea_orm::DbErr::Custom("database health check returned no rows".to_owned()))?;
    Ok(())
}
