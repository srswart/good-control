//! Common error types for the application.

use thiserror::Error;

/// Application-wide error type.
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Server error.
    #[error("Server error: {0}")]
    Server(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

