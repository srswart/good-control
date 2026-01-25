//! @arrive component: graphql
//! GraphQL schema and resolvers (API surface).
//!
//! This crate provides:
//! - GraphQL schema with Query and Mutation root types
//! - Resolvers for authentication, users, and health checks
//! - Request context with user session and database pool
//! - Integration with auth middleware

mod context;
mod mutation;
mod query;
mod schema;
mod types;

pub use context::GraphQLContext;
pub use schema::{build_schema, GraphQLSchema};

