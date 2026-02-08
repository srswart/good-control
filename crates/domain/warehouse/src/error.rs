//! Warehouse domain error types.

use thiserror::Error;

/// Warehouse domain error.
#[derive(Debug, Error)]
pub enum WarehouseError {
    /// Invalid state transition.
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Invalid quantity.
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    /// Discrepancy validation failed.
    #[error("Discrepancy validation failed: {0}")]
    DiscrepancyValidationFailed(String),

    /// QC outcome required.
    #[error("QC outcome required before completing receiving task")]
    QCOutcomeRequired,

    /// Invalid receiving line.
    #[error("Invalid receiving line: {0}")]
    InvalidReceivingLine(String),

    /// Quarantine operation failed.
    #[error("Quarantine operation failed: {0}")]
    QuarantineOperationFailed(String),
}

/// Result type alias for warehouse operations.
pub type Result<T> = std::result::Result<T, WarehouseError>;
