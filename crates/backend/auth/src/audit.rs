//! Audit event recording and retrieval.

use crate::error::Result;
use persistence::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Audit event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// User authentication events.
    AuthLogin,
    AuthLogout,
    AuthFailed,

    /// User management events.
    UserCreated,
    UserUpdated,
    UserDeactivated,
    UserActivated,

    /// Role and permission events.
    RoleAssigned,
    RoleRemoved,

    /// Inventory events.
    InventoryAdjusted,
    InventoryScrapped,

    /// Compliance events.
    ComplianceApproved,
    ComplianceRejected,

    /// System events.
    SystemConfigChanged,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AuthLogin => "auth_login",
            Self::AuthLogout => "auth_logout",
            Self::AuthFailed => "auth_failed",
            Self::UserCreated => "user_created",
            Self::UserUpdated => "user_updated",
            Self::UserDeactivated => "user_deactivated",
            Self::UserActivated => "user_activated",
            Self::RoleAssigned => "role_assigned",
            Self::RoleRemoved => "role_removed",
            Self::InventoryAdjusted => "inventory_adjusted",
            Self::InventoryScrapped => "inventory_scrapped",
            Self::ComplianceApproved => "compliance_approved",
            Self::ComplianceRejected => "compliance_rejected",
            Self::SystemConfigChanged => "system_config_changed",
        };
        write!(f, "{}", s)
    }
}

/// Audit event model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditEvent {
    pub id: i64,
    pub user_id: Option<i64>,
    pub event_type: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub created_at: String,
}

/// Record an audit event.
///
/// # Arguments
///
/// * `pool` - Database pool
/// * `user_id` - Optional user ID (None for system events)
/// * `event_type` - Type of event
/// * `action` - Action description
/// * `resource_type` - Optional resource type (e.g., "user", "inventory")
/// * `resource_id` - Optional resource ID
/// * `details` - Optional JSON details
/// * `ip_address` - Optional IP address
/// * `user_agent` - Optional user agent
/// * `request_id` - Optional request/correlation ID
pub async fn record_audit_event(
    pool: &DbPool,
    user_id: Option<i64>,
    event_type: AuditEventType,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    details: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    request_id: Option<&str>,
) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO audit_events (user_id, event_type, resource_type, resource_id, action, details, ip_address, user_agent, request_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(event_type.to_string())
    .bind(resource_type)
    .bind(resource_id)
    .bind(action)
    .bind(details)
    .bind(ip_address)
    .bind(user_agent)
    .bind(request_id)
    .execute(pool.inner())
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get audit events for a user.
pub async fn get_user_audit_events(
    pool: &DbPool,
    user_id: i64,
    limit: Option<i64>,
) -> Result<Vec<AuditEvent>> {
    let limit = limit.unwrap_or(100);

    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT id, user_id, event_type, resource_type, resource_id, action, details, ip_address, user_agent, request_id, created_at
         FROM audit_events
         WHERE user_id = ?
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool.inner())
    .await?;

    Ok(events)
}

/// Get audit events by type.
pub async fn get_audit_events_by_type(
    pool: &DbPool,
    event_type: AuditEventType,
    limit: Option<i64>,
) -> Result<Vec<AuditEvent>> {
    let limit = limit.unwrap_or(100);

    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT id, user_id, event_type, resource_type, resource_id, action, details, ip_address, user_agent, request_id, created_at
         FROM audit_events
         WHERE event_type = ?
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(event_type.to_string())
    .bind(limit)
    .fetch_all(pool.inner())
    .await?;

    Ok(events)
}

/// Get audit events for a resource.
pub async fn get_resource_audit_events(
    pool: &DbPool,
    resource_type: &str,
    resource_id: &str,
    limit: Option<i64>,
) -> Result<Vec<AuditEvent>> {
    let limit = limit.unwrap_or(100);

    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT id, user_id, event_type, resource_type, resource_id, action, details, ip_address, user_agent, request_id, created_at
         FROM audit_events
         WHERE resource_type = ? AND resource_id = ?
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(resource_type)
    .bind(resource_id)
    .bind(limit)
    .fetch_all(pool.inner())
    .await?;

    Ok(events)
}

/// Get recent audit events.
pub async fn get_recent_audit_events(
    pool: &DbPool,
    limit: Option<i64>,
) -> Result<Vec<AuditEvent>> {
    let limit = limit.unwrap_or(100);

    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT id, user_id, event_type, resource_type, resource_id, action, details, ip_address, user_agent, request_id, created_at
         FROM audit_events
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool.inner())
    .await?;

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::{SqliteUserRepository, UserRepository};
    use persistence::DbPoolConfig;

    async fn create_test_pool() -> DbPool {
        let config = DbPoolConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let pool = DbPool::new(&config).await.unwrap();

        // Run migrations
        sqlx::migrate!("../../../migrations")
            .run(pool.inner())
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_record_audit_event() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        let event_id = record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in",
            None,
            None,
            Some(r#"{"method": "password"}"#),
            Some("127.0.0.1"),
            Some("Test Agent"),
            Some("req-123"),
        ).await.unwrap();

        assert!(event_id > 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_audit_events() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Record multiple events
        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in",
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();

        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::UserUpdated,
            "User updated profile",
            Some("user"),
            Some(&user.id.to_string()),
            None,
            None,
            None,
            None,
        ).await.unwrap();

        // Get events
        let events = get_user_audit_events(&pool, user.id, None).await.unwrap();
        assert_eq!(events.len(), 2);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_audit_events_by_type() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Record events of different types
        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in",
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();

        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in again",
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();

        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::UserUpdated,
            "User updated",
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();

        // Get login events
        let events = get_audit_events_by_type(&pool, AuditEventType::AuthLogin, None).await.unwrap();
        assert_eq!(events.len(), 2);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_resource_audit_events() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Record events for a specific resource
        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            "Adjusted quantity",
            Some("inventory"),
            Some("item-123"),
            None,
            None,
            None,
            None,
        ).await.unwrap();

        record_audit_event(
            &pool,
            Some(user.id),
            AuditEventType::InventoryScrapped,
            "Scrapped item",
            Some("inventory"),
            Some("item-123"),
            None,
            None,
            None,
            None,
        ).await.unwrap();

        // Get events for the resource
        let events = get_resource_audit_events(&pool, "inventory", "item-123", None).await.unwrap();
        assert_eq!(events.len(), 2);

        pool.close().await;
    }
}

