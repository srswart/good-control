//! @arrive component: assets-ui
//! Assets UI: drones and batteries lifecycle views.
//!
//! This crate provides UI components for asset management:
//! - Drones registry and lifecycle views
//! - Batteries registry and lifecycle views

pub mod drones;
pub mod batteries;

pub use drones::DronesView;
pub use batteries::BatteriesView;
