//! @arrive component: procurement
//! Procurement domain: suppliers, purchase orders, inbound documents.
//!
//! This crate provides the procurement domain models and operations:
//! - Supplier management
//! - Purchase order lifecycle
//! - Inbound document handling

pub mod error;
pub mod supplier;
pub mod purchase_order;
pub mod inbound_document;

pub use error::{ProcurementError, Result};
pub use supplier::Supplier;
pub use purchase_order::{PurchaseOrder, PurchaseOrderStatus, PurchaseOrderLine};
pub use inbound_document::{InboundDocument, InboundDocumentType, InboundDocumentStatus};
