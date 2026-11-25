use serde::Serialize;
use thiserror::Error;

/// Structured, safe error payload serialized across Tauri IPC.
///
/// Never exposes raw OS paths, SQL statements, or internal stack traces.
#[derive(Debug, Serialize)]
struct SerializedAppError {
    code: &'static str,
    message: String,
}

/// Unified application error type for backend services and Tauri IPC commands.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Storage unavailable: {0}")]
    StorageUnavailable(String),

    #[error("Database initialization failed: {0}")]
    DatabaseInit(String),

    #[error("Database migration failed: {0}")]
    MigrationFailed(String),

    #[error("Database integrity check failed or file is corrupted: {0}")]
    DatabaseCorrupted(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Invalid path or identifier: {0}")]
    InvalidPath(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Stable machine-readable error code for IPC consumers.
    pub fn code(&self) -> &'static str {
        match self {
            Self::StorageUnavailable(_) => "STORAGE_UNAVAILABLE",
            Self::DatabaseInit(_) => "DATABASE_INIT_FAILED",
            Self::MigrationFailed(_) => "DATABASE_MIGRATION_FAILED",
            Self::DatabaseCorrupted(_) => "DATABASE_CORRUPTED",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Config(_) => "CONFIG_ERROR",
            Self::Platform(_) => "PLATFORM_ERROR",
            Self::InvalidPath(_) => "INVALID_PATH",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Safe user-facing message that omits sensitive paths and internal details.
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation(msg) | Self::AlreadyExists(msg) | Self::NotFound(msg) => msg.clone(),
            Self::StorageUnavailable(_) => {
                "Unable to access a writable application data directory.".to_string()
            }
            Self::DatabaseInit(_) => "Unable to initialize the local database.".to_string(),
            Self::MigrationFailed(_) => {
                "Unable to update the local database schema. Existing data was not overwritten."
                    .to_string()
            }
            Self::DatabaseCorrupted(_) => {
                "The local database file appears corrupted or unreadable.".to_string()
            }
            Self::Database(_) => "A local database operation failed.".to_string(),
            Self::Io(_) => "A local file operation failed.".to_string(),
            Self::Config(_) => "Application configuration is invalid.".to_string(),
            Self::Platform(_) => "A platform integration operation failed.".to_string(),
            Self::InvalidPath(_) => "The requested file or directory name is invalid.".to_string(),
            Self::Internal(_) => "An unexpected internal error occurred.".to_string(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializedAppError {
            code: self.code(),
            message: self.user_message(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
