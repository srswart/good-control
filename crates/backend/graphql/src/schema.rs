//! GraphQL schema builder.

use async_graphql::{EmptySubscription, Schema};

use crate::mutation::Mutation;
use crate::query::Query;

/// GraphQL schema type.
pub type GraphQLSchema = Schema<Query, Mutation, EmptySubscription>;

/// Build the GraphQL schema.
pub fn build_schema() -> GraphQLSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .finish()
}

