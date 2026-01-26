//! @arrive component: warehouse-ui
//! Warehouse UI: receiving, putaway, pick/pack/ship; scan-first.
//!
//! This crate provides UI components for warehouse management:
//! - Receiving workflow
//! - Putaway tasks
//! - Pick tasks
//! - Pack tasks
//! - Ship tasks

pub mod receiving;
pub mod putaway;
pub mod pick;
pub mod pack;
pub mod ship;

pub use receiving::ReceivingView;
pub use putaway::PutawayView;
pub use pick::PickView;
pub use pack::PackView;
pub use ship::ShipView;
