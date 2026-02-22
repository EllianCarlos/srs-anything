use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Repository failure: {0}")]
    Message(String),
    #[error("Database failure: {0}")]
    Database(#[from] sea_orm::DbErr),
}
