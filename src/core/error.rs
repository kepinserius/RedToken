use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedTokenError {
    #[error("Failed to read file: {path}")]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    #[error("Token validation failed: {0}")]
    TokenValidationError(String),

    #[error("Token with ID {0} not found")]
    TokenNotFound(String),

    #[error("Failed to send notification: {0}")]
    NotificationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("API error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unauthorized access: {0}")]
    UnauthorizedError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type RedTokenResult<T> = Result<T, RedTokenError>;
