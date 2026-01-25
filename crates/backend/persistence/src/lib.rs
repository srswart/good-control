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
mod pool;
mod repository;
mod transaction;

pub use error::{PersistenceError, Result};
pub use inventory_repository::{
    InventoryMovementRecord, InventoryPositionRecord, InventoryRepository, ItemRecord,
    LocationRecord, SqliteInventoryRepository,
};
pub use inventory_service::InventoryService;
pub use migrations::run_migrations;
pub use pool::{DbPool, DbPoolConfig};
pub use repository::Repository;
pub use transaction::begin_transaction;
