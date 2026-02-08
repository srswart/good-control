//! GraphQL client layer with standardized error handling.
//!
//! This module provides:
//! - Typed GraphQL client for making requests to the backend
//! - Error mapping from backend StructuredError to frontend error types
//! - Request correlation IDs for tracing
//! - Standardized error handling for auth, validation, and compliance blocks

pub mod client;
pub mod auth_token;
pub mod endpoint;
pub mod error;

pub use client::GraphQLClient;
pub use auth_token::{auth_token, set_auth_token};
pub use endpoint::graphql_endpoint;
pub use error::{ErrorCategory, FieldError, GraphQLError, StructuredError};

