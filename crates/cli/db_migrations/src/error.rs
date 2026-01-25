//! Error types for database migrations.

use thiserror::Error;

/// Result type for database migration operations.
pub type Result<T> = std::result::Result<T, DbMigrationError>;

/// Errors that can occur during database migration operations.
#[derive(Debug, Error)]
pub enum DbMigrationError {
    /// Database connection error.
    #[error("database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    /// Migration error.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Backup error.
    #[error("backup error: {0}")]
    Backup(String),

    /// Restore error.
    #[error("restore error: {0}")]
    Restore(String),

    /// Verification error.
    #[error("verification error: {0}")]
    Verification(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid database path.
    #[error("invalid database path: {0}")]
    InvalidPath(String),
}

