//! @arrive component: foundation
//! CLI foundation: clap scaffolding, configuration, logging, and output conventions.
//!
//! This crate provides the foundational infrastructure for the CLI:
//! - Configuration loading and validation
//! - Logging/tracing initialization
//! - Output formatting (human-readable and JSON)
//! - Common utilities for CLI commands

pub mod config;
pub mod logging;
pub mod output;

pub use config::CliConfig;
pub use logging::init_logging;
pub use output::{OutputFormat, OutputWriter};

