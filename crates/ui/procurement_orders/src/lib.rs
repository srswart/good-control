//! @arrive component: procurement-orders-ui
//! Procurement + Orders UI: POs, allocations, fulfillment.
//!
//! This crate provides UI components for procurement and orders management:
//! - Purchase Orders (POs)
//! - Allocations
//! - Fulfillment

pub mod purchase_orders;
pub mod allocations;
pub mod fulfillment;

pub use purchase_orders::PurchaseOrdersView;
pub use allocations::AllocationsView;
pub use fulfillment::FulfillmentView;
