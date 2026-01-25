//! @arrive component: persistence
//!
//! Error types for the persistence layer.

use thiserror::Error;

/// Result type for persistence operations.
pub type Result<T> = std::result::Result<T, PersistenceError>;

/// Errors that can occur in the persistence layer.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Database connection error.
    #[error("database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    /// Migration error.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Entity not found.
    #[error("entity not found: {entity_type} with id {id}")]
    NotFound {
        entity_type: &'static str,
        id: String,
    },

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Generic error helper for simple messages.
    #[error("{0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error_display() {
        let err = PersistenceError::NotFound {
            entity_type: "User",
            id: "123".to_string(),
        };
        assert_eq!(err.to_string(), "entity not found: User with id 123");
    }

    #[test]
    fn test_configuration_error_display() {
        let err = PersistenceError::Configuration("invalid database URL".to_string());
        assert_eq!(err.to_string(), "configuration error: invalid database URL");
    }
}
