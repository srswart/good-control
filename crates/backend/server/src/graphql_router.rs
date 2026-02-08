//! GraphQL router configuration.

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use auth::{AuthLayer, CurrentUser, optional_auth_middleware};
use graphql::{build_schema, GraphQLContext, GraphQLSchema};
use persistence::DbPool;
use serde_json::Value;
use std::sync::Arc;

/// GraphQL state containing schema and database pool.
#[derive(Clone)]
struct GraphQLState {
    schema: GraphQLSchema,
    pool: DbPool,
}

/// GraphQL request.
#[derive(serde::Deserialize)]
struct GraphQLRequest {
    query: String,
    #[serde(default)]
    variables: Option<Value>,
    #[serde(default)]
    operation_name: Option<String>,
}

/// GraphQL handler.
async fn graphql_handler(
    State(state): State<GraphQLState>,
    current_user: Option<Extension<CurrentUser>>,
    Json(req): Json<GraphQLRequest>,
) -> Response {
    let context = match current_user {
        Some(Extension(current_user)) => {
            GraphQLContext::with_user(state.pool.clone(), current_user.user)
        }
        None => GraphQLContext::new(state.pool.clone()),
    };

    let mut request = async_graphql::Request::new(req.query);

    if let Some(variables) = req.variables {
        request = request.variables(async_graphql::Variables::from_json(variables));
    }

    if let Some(operation_name) = req.operation_name {
        request = request.operation_name(operation_name);
    }

    let response = state.schema.execute(request.data(context)).await;

    (StatusCode::OK, Json(response)).into_response()
}

/// GraphQL playground handler.
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

/// Create GraphQL routes.
pub fn routes(pool: DbPool) -> Router {
    let schema = build_schema();
    let state = GraphQLState { schema, pool };
    let auth_layer = Arc::new(AuthLayer::new(state.pool.clone()));

    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .layer(axum::middleware::from_fn(optional_auth_middleware))
        .layer(Extension(auth_layer))
        .with_state(state)
}

