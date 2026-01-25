//! @arrive component: logistics
//! Logistics domain: shipments, tracking, and 3PL abstractions.
//!
//! This crate provides the logistics domain models and operations:
//! - Shipment management and lifecycle
//! - Tracking events and status timeline
//! - 3PL integration abstractions

pub mod error;
pub mod shipment;
pub mod tracking;

pub use error::{LogisticsError, Result};
pub use shipment::{Shipment, ShipmentStatus};
pub use tracking::{TrackingEvent, TrackingEventType};
