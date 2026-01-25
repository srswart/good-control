//! GraphQL Mutation root.

use async_graphql::*;
use auth::{authenticate, AuditEventType, SqliteUserRepository, UserRepository, record_audit_event};

use crate::context::GraphQLContext;
use crate::types::{LoginResponse, User};

/// GraphQL Mutation root.
#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    /// Login with username and password.
    async fn login(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> Result<LoginResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());

        // Authenticate user
        let (user, session) = authenticate(
            &user_repo,
            &gql_ctx.pool,
            &username,
            &password,
            None, // IP address - could be extracted from request
            None, // User agent - could be extracted from request
        )
        .await
        .map_err(|e| Error::new(format!("Authentication failed: {}", e)))?;

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in via GraphQL",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(LoginResponse {
            user: User::from(user),
            token: session.token,
        })
    }

    /// Logout (invalidate current session).
    async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::AuthLogout,
            "User logged out via GraphQL",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        // Note: In a real implementation, we would invalidate the session token here
        // For now, we just record the audit event
        Ok(true)
    }

    /// Create a new user (requires admin.users permission).
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        let user = user_repo.create(&username, &email, &password)
            .await
            .map_err(|e| Error::new(format!("Failed to create user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserCreated,
            &format!("Created user: {}", username),
            Some("user"),
            Some(&user.id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(User::from(user))
    }

    /// Deactivate a user (requires admin.users permission).
    async fn deactivate_user(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        user_repo.deactivate(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to deactivate user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserDeactivated,
            &format!("Deactivated user ID: {}", user_id),
            Some("user"),
            Some(&user_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    /// Activate a user (requires admin.users permission).
    async fn activate_user(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        user_repo.activate(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to activate user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserActivated,
            &format!("Activated user ID: {}", user_id),
            Some("user"),
            Some(&user_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }
}

