//! @arrive component: logistics-ui
//! Logistics UI: shipments, tracking, cross-border pack review.
//!
//! This crate provides UI components for logistics management:
//! - Shipments management
//! - Tracking information
//! - Cross-border pack review

pub mod shipments;
pub mod tracking;
pub mod cross_border_pack_review;

pub use shipments::ShipmentsView;
pub use tracking::TrackingView;
pub use cross_border_pack_review::CrossBorderPackReviewView;
