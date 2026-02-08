//! @arrive component: inventory-ui
//! Inventory UI: items, stock, traceability, cycle counts.
//!
//! This crate provides UI components for inventory management:
//! - Items list and detail views
//! - Stock/inventory positions
//! - Traceability (movement history)
//! - Cycle counts
//! - GraphQL client demo

pub mod items;
pub mod stock;
pub mod traceability;
pub mod cycle_counts;
pub mod graphql_demo;
pub mod i18n;

pub use items::ItemsList;
pub use stock::StockView;
pub use traceability::TraceabilityView;
pub use cycle_counts::CycleCountsView;
pub use graphql_demo::GraphQLDemo;
