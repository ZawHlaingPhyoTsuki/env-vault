use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("record not found".into()),
            rusqlite::Error::SqliteFailure(err, Some(msg))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Conflict(msg)
            }
            rusqlite::Error::SqliteFailure(err, None)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Conflict("constraint violation".into())
            }
            other => AppError::Database(other.to_string()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
