//! @arrive component: foundation
//! Frontend foundation: app bootstrap, theme, Tailwind integration, error boundary.
//!
//! This crate provides the foundational infrastructure for the frontend:
//! - App bootstrap and initialization
//! - Error boundary component
//! - Theme setup (to be expanded in future advances)

pub mod error_boundary;

pub use error_boundary::{BoundaryError, ErrorBoundary};
