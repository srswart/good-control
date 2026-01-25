//! GraphQL Query root.

use async_graphql::*;
use auth::{SqliteUserRepository, UserRepository};

use crate::context::GraphQLContext;
use crate::types::{HealthResponse, User};

/// GraphQL Query root.
#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    /// Health check query.
    async fn health(&self, ctx: &Context<'_>) -> Result<HealthResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Test database connection
        let db_status = match sqlx::query("SELECT 1")
            .fetch_one(gql_ctx.pool.inner())
            .await
        {
            Ok(_) => "connected",
            Err(_) => "disconnected",
        };

        Ok(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: db_status.to_string(),
        })
    }

    /// Get the current authenticated user.
    async fn current_user(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let auth_user = gql_ctx.require_user()?;
        Ok(User::from(auth_user.clone()))
    }

    /// Get a user by ID (requires admin.users permission).
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        let user = user_repo.find_by_id(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to find user: {}", e)))?;

        Ok(user.map(User::from))
    }

    /// List all users (requires admin.users permission).
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let users = sqlx::query_as::<_, auth::User>(
            "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
             FROM users 
             ORDER BY username"
        )
        .fetch_all(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to list users: {}", e)))?;

        Ok(users.into_iter().map(User::from).collect())
    }
}

