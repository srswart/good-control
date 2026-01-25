//! @arrive component: db-migrations
//! CLI database migrations, backup, and restore functionality.
//!
//! This crate provides:
//! - Database migration runner with dry-run support
//! - SQLite backup functionality
//! - SQLite restore functionality
//! - Database integrity verification

mod backup;
mod error;
mod migrate;
mod restore;
mod verify;

pub use backup::backup_database;
pub use error::{DbMigrationError, Result};
pub use migrate::{list_migrations, run_migrations, MigrationInfo};
pub use restore::restore_database;
pub use verify::verify_database;

