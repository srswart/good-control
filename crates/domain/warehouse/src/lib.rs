//! @arrive component: warehouse
//! Warehouse domain: receiving, putaway, picking, packing, shipping.
//!
//! This crate provides the warehouse domain models and operations:
//! - Receiving workflows with discrepancy capture
//! - Quality control (QC) outcomes
//! - Quarantine and release transitions
//! - Picking workflows with exception handling
//! - Packing workflows with cartonization
//! - Integration with inventory ledger

pub mod error;
pub mod receiving;
pub mod picking;
pub mod packing;
pub mod shipping;

pub use error::{WarehouseError, Result};
pub use receiving::{
    ReceivingTask, ReceivingTaskStatus, ReceivingDiscrepancy, DiscrepancyType,
    QCOutcome, QCResult, QuarantineStatus, ReceivingLine,
};
pub use picking::{
    PickTask, PickTaskStatus, PickTaskType, PickLine, PickException, PickExceptionType,
};
pub use packing::{
    PackTask, PackTaskStatus, PackLine, Package, PackedItem,
};
pub use shipping::{
    ShipTask, ShipTaskStatus, ShipLine, ComplianceBlock, ComplianceBlockCategory,
    BlockReason, ShipTaskStatistics,
};
