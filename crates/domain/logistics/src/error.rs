//! Error types for the logistics domain.

use thiserror::Error;

/// Result type for logistics operations.
pub type Result<T> = std::result::Result<T, LogisticsError>;

/// Errors that can occur in the logistics domain.
#[derive(Debug, Error)]
pub enum LogisticsError {
    /// Shipment not found.
    #[error("Shipment not found: {0}")]
    ShipmentNotFound(String),

    /// Package not found.
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    /// Invalid shipment state transition.
    #[error("Invalid shipment state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Invalid tracking event.
    #[error("Invalid tracking event: {0}")]
    InvalidTrackingEvent(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error.
    #[error("{0}")]
    Generic(String),
}

