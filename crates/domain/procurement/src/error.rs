//! Error types for the procurement domain.

use thiserror::Error;

/// Result type for procurement operations.
pub type Result<T> = std::result::Result<T, ProcurementError>;

/// Errors that can occur in the procurement domain.
#[derive(Debug, Error)]
pub enum ProcurementError {
    /// Supplier not found.
    #[error("Supplier not found: {0}")]
    SupplierNotFound(String),

    /// Purchase order not found.
    #[error("Purchase order not found: {0}")]
    PurchaseOrderNotFound(String),

    /// Inbound document not found.
    #[error("Inbound document not found: {0}")]
    InboundDocumentNotFound(String),

    /// Invalid purchase order state transition.
    #[error("Invalid purchase order state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Invalid purchase order line.
    #[error("Invalid purchase order line: {0}")]
    InvalidPurchaseOrderLine(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error.
    #[error("{0}")]
    Generic(String),
}
