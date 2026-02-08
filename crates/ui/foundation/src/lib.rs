//! @arrive component: foundation
//! Frontend foundation: app bootstrap, theme, Tailwind integration, error boundary.
//!
//! This crate provides the foundational infrastructure for the frontend:
//! - App bootstrap and initialization
//! - Error boundary component
//! - GraphQL client layer with error mapping
//! - Theme setup (to be expanded in future advances)

pub mod error_boundary;
pub mod graphql;

pub use error_boundary::{BoundaryError, ErrorBoundary};
pub use graphql::{
    auth_token, graphql_endpoint, set_auth_token, ErrorCategory, FieldError, GraphQLClient,
    GraphQLError, StructuredError,
};
