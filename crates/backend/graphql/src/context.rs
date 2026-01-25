//! GraphQL request context.

use auth::User;
use persistence::DbPool;

/// GraphQL request context containing user session and database pool.
#[derive(Clone)]
pub struct GraphQLContext {
    /// Database pool for queries and mutations.
    pub pool: DbPool,
    /// Current authenticated user (if any).
    pub current_user: Option<User>,
}

impl GraphQLContext {
    /// Create a new GraphQL context.
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            current_user: None,
        }
    }

    /// Create a new GraphQL context with an authenticated user.
    pub fn with_user(pool: DbPool, user: User) -> Self {
        Self {
            pool,
            current_user: Some(user),
        }
    }

    /// Get the current user, returning an error if not authenticated.
    pub fn require_user(&self) -> async_graphql::Result<&User> {
        self.current_user
            .as_ref()
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))
    }

    /// Check if the current user has a specific permission.
    pub async fn check_permission(&self, permission: &str) -> async_graphql::Result<bool> {
        let user = self.require_user()?;
        auth::check_permission(&self.pool, user.id, permission)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Permission check failed: {}", e)))
    }

    /// Require the current user to have a specific permission.
    pub async fn require_permission(&self, permission: &str) -> async_graphql::Result<()> {
        let user = self.require_user()?;
        auth::require_permission(&self.pool, user.id, permission)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Permission denied: {}", e)))
    }
}

