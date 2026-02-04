use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("database error: {0}")]
    Database(#[from] store::DbErr),

    #[error("member not found")]
    MemberNotFound,

    #[error("username already exists")]
    UsernameExists,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

