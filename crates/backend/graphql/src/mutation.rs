//! GraphQL Mutation root.

use async_graphql::*;
use auth::{authenticate, AuditEventType, SqliteUserRepository, UserRepository, record_audit_event};

use crate::context::GraphQLContext;
use crate::types::{CycleCountLine, CycleCountSession, LoginResponse, ReserveResponse, User};
use persistence::InventoryService;

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

    /// Reserve inventory, optionally allowing partial allocation.
    async fn reserve_inventory(
        &self,
        ctx: &Context<'_>,
        sku: String,
        location_id: String,
        quantity: i64,
        #[graphql(default = false)] allow_partial: bool,
        reference: Option<String>,
    ) -> Result<ReserveResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let (movement_id, allocated) = service
            .reserve_inventory_with_partial(
                &sku,
                &location_id,
                quantity,
                allow_partial,
                &user.username,
                reference.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Reservation failed: {}", e)))?;

        Ok(ReserveResponse {
            movement_id,
            allocated,
        })
    }

    /// Start a cycle count session with given ABC class (A/B/C).
    async fn start_cycle_count(&self, ctx: &Context<'_>, abc_class: String) -> Result<CycleCountSession> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        let session_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO cycle_count_sessions (id, abc_class, status, created_at)
               VALUES (?, ?, 'open', datetime('now'))"#,
        )
        .bind(&session_id)
        .bind(&abc_class)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to create cycle count session: {}", e)))?;

        // Return session
        let session = sqlx::query_as::<_, CycleCountSession>(
            r#"SELECT id, abc_class, status, created_at, completed_at FROM cycle_count_sessions WHERE id = ?"#,
        )
        .bind(&session_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load created session: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Started cycle count session {} ({})", session_id, abc_class),
            Some("inventory"),
            Some(&session_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(session)
    }

    /// Record a cycle count for a SKU/location within a session.
    async fn record_cycle_count(
        &self,
        ctx: &Context<'_>,
        session_id: String,
        sku: String,
        location_id: String,
        counted_qty: i64,
    ) -> Result<CycleCountLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Determine expected from inventory_positions
        let expected: Option<i64> = sqlx::query_scalar(
            r#"SELECT quantity_available FROM inventory_positions
               WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL"#,
        )
        .bind(&sku)
        .bind(&location_id)
        .fetch_optional(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to read expected quantity: {}", e)))?;
        let expected_qty = expected.unwrap_or(0);
        let variance = counted_qty - expected_qty;

        // Upsert line
        let line_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO cycle_count_lines
               (id, session_id, sku, location_id, expected_qty, counted_qty, variance, status)
               VALUES (?, ?, ?, ?, ?, ?, ?, 'counted')"#,
        )
        .bind(&line_id)
        .bind(&session_id)
        .bind(&sku)
        .bind(&location_id)
        .bind(expected_qty)
        .bind(counted_qty)
        .bind(variance)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to write cycle count line: {}", e)))?;

        // Audit
        let details = format!("{{\"sku\":\"{}\",\"loc\":\"{}\",\"expected\":{},\"counted\":{},\"variance\":{}}}", sku, location_id, expected_qty, counted_qty, variance);
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            "Cycle count recorded",
            Some("inventory"),
            Some(&session_id),
            Some(&details),
            None,
            None,
            None,
        )
        .await;

        // Return line
        let line = sqlx::query_as::<_, CycleCountLine>(
            r#"SELECT id, session_id, sku, location_id, expected_qty, counted_qty, variance, status
               FROM cycle_count_lines WHERE id = ?"#,
        )
        .bind(&line_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load cycle count line: {}", e)))?;

        Ok(line)
    }

    /// Complete a cycle count session.
    async fn complete_cycle_count(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        sqlx::query(
            r#"UPDATE cycle_count_sessions SET status = 'completed', completed_at = datetime('now') WHERE id = ?"#,
        )
        .bind(&session_id)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to complete cycle count session: {}", e)))?;

        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            "Cycle count session completed",
            Some("inventory"),
            Some(&session_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }
}
