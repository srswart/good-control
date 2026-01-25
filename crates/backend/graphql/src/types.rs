//! GraphQL types and models.

use async_graphql::*;
use auth::{Permission as AuthPermission, Role as AuthRole, User as AuthUser};

/// Role type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Role {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AuthRole> for Role {
    fn from(role: AuthRole) -> Self {
        Self {
            id: ID(role.id.to_string()),
            name: role.name,
            description: role.description,
            created_at: role.created_at,
            updated_at: role.updated_at,
        }
    }
}

/// Permission type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Permission {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub created_at: String,
}

impl From<AuthPermission> for Permission {
    fn from(permission: AuthPermission) -> Self {
        Self {
            id: ID(permission.id.to_string()),
            name: permission.name,
            description: permission.description,
            resource: permission.resource,
            action: permission.action,
            created_at: permission.created_at,
        }
    }
}

/// User type for GraphQL.
#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[ComplexObject]
impl User {
    /// Get the roles assigned to this user.
    async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let gql_ctx = ctx.data::<crate::GraphQLContext>()?;
        let user_id: i64 = self.id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let auth_roles = auth::get_user_roles(&gql_ctx.pool, user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to get user roles: {}", e)))?;

        Ok(auth_roles.into_iter().map(Role::from).collect())
    }

    /// Get the permissions assigned to this user.
    async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let gql_ctx = ctx.data::<crate::GraphQLContext>()?;
        let user_id: i64 = self.id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let auth_permissions = auth::get_user_permissions(&gql_ctx.pool, user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to get user permissions: {}", e)))?;

        Ok(auth_permissions.into_iter().map(Permission::from).collect())
    }
}

impl From<AuthUser> for User {
    fn from(user: AuthUser) -> Self {
        Self {
            id: ID(user.id.to_string()),
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Login response containing user and session token.
#[derive(SimpleObject)]
pub struct LoginResponse {
    /// The authenticated user.
    pub user: User,
    /// Session token for subsequent requests.
    pub token: String,
}

/// Health check response.
#[derive(SimpleObject)]
pub struct HealthResponse {
    /// Status of the service.
    pub status: String,
    /// Version of the service.
    pub version: String,
    /// Database connection status.
    pub database: String,
}

/// Reserve inventory response.
#[derive(SimpleObject)]
pub struct ReserveResponse {
    /// Movement ID created for this reservation.
    pub movement_id: String,
    /// Quantity actually allocated (may be less than requested when allowPartial=true).
    pub allocated: i64,
}
