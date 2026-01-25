//! @arrive component: persistence
//!
//! Persistence layer for the drone-inventory system.
//!
//! This crate provides:
//! - Database connection pool management
//! - Migration runner
//! - Repository traits and implementations

mod error;
mod migrations;
mod pool;
mod repository;

pub use error::{PersistenceError, Result};
pub use migrations::run_migrations;
pub use pool::{DbPool, DbPoolConfig};
pub use repository::Repository;

