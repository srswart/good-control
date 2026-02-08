//! @arrive component: persistence
//!
//! Persistence layer for the drone-inventory system.
//!
//! This crate provides:
//! - Database connection pool management
//! - Migration runner
//! - Repository traits and implementations

mod error;
mod inventory_repository;
mod inventory_service;
mod migrations;
mod mock_data;
mod pool;
mod repository;
mod transaction;
mod warehouse_service;

pub use error::{PersistenceError, Result};
pub use inventory_repository::{
    get_current_position, get_lot_genealogy_by_number, get_lot_shipments_by_number,
    get_serial_genealogy, get_serial_shipments, InventoryMovementRecord, InventoryPositionRecord,
    InventoryRepository, ItemRecord, LocationRecord, SqliteInventoryRepository,
};
pub use inventory_service::InventoryService;
pub use mock_data::{reload_mock_data, MockDataSummary};
pub use migrations::run_migrations;
pub use pool::{DbPool, DbPoolConfig};
pub use repository::Repository;
pub use transaction::begin_transaction;
pub use warehouse_service::WarehouseService;
