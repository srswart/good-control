//! Error types for authentication and authorization.

use thiserror::Error;

/// Result type for auth operations.
pub type Result<T> = std::result::Result<T, AuthError>;

/// Errors that can occur during authentication and authorization.
#[derive(Debug, Error)]
pub enum AuthError {
    /// User not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// Invalid credentials.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Invalid token.
    #[error("invalid token")]
    InvalidToken,

    /// Token expired.
    #[error("token expired")]
    TokenExpired,

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Role not found.
    #[error("role not found: {0}")]
    RoleNotFound(String),

    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Persistence error.
    #[error("persistence error: {0}")]
    Persistence(#[from] persistence::PersistenceError),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),
}

