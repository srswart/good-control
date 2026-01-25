//! @arrive component: foundation
//! Backend foundation: configuration, tracing, and core utilities.
//!
//! This crate provides the foundational infrastructure for the backend:
//! - Configuration loading and validation
//! - Tracing/logging initialization
//! - Common utilities

pub mod config;
pub mod tracing_init;

pub use config::{AppConfig, ServerConfig};
pub use tracing_init::init_tracing;

