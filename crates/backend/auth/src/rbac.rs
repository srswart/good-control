//! Role-Based Access Control (RBAC) models and logic.

use crate::error::{AuthError, Result};
use persistence::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Role model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Permission model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub created_at: String,
}

/// Check if a user has a specific permission.
///
/// # Arguments
///
/// * `pool` - Database pool
/// * `user_id` - User ID to check
/// * `permission_name` - Permission name to check (e.g., "inventory.read")
///
/// # Returns
///
/// True if the user has the permission, false otherwise.
pub async fn check_permission(pool: &DbPool, user_id: i64, permission_name: &str) -> Result<bool> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles ur
         JOIN role_permissions rp ON ur.role_id = rp.role_id
         JOIN permissions p ON rp.permission_id = p.id
         WHERE ur.user_id = ? AND p.name = ?"
    )
    .bind(user_id)
    .bind(permission_name)
    .fetch_one(pool.inner())
    .await?;

    Ok(result > 0)
}

/// Check if a user has a specific permission, returning an error if not.
pub async fn require_permission(pool: &DbPool, user_id: i64, permission_name: &str) -> Result<()> {
    if check_permission(pool, user_id, permission_name).await? {
        Ok(())
    } else {
        Err(AuthError::PermissionDenied(permission_name.to_string()))
    }
}

/// Get all permissions for a user.
pub async fn get_user_permissions(pool: &DbPool, user_id: i64) -> Result<Vec<Permission>> {
    let permissions = sqlx::query_as::<_, Permission>(
        "SELECT DISTINCT p.id, p.name, p.description, p.resource, p.action, p.created_at
         FROM permissions p
         JOIN role_permissions rp ON p.id = rp.permission_id
         JOIN user_roles ur ON rp.role_id = ur.role_id
         WHERE ur.user_id = ?
         ORDER BY p.name"
    )
    .bind(user_id)
    .fetch_all(pool.inner())
    .await?;

    Ok(permissions)
}

/// Get all roles for a user.
pub async fn get_user_roles(pool: &DbPool, user_id: i64) -> Result<Vec<Role>> {
    let roles = sqlx::query_as::<_, Role>(
        "SELECT r.id, r.name, r.description, r.created_at, r.updated_at
         FROM roles r
         JOIN user_roles ur ON r.id = ur.role_id
         WHERE ur.user_id = ?
         ORDER BY r.name"
    )
    .bind(user_id)
    .fetch_all(pool.inner())
    .await?;

    Ok(roles)
}

/// Assign a role to a user.
pub async fn assign_role(pool: &DbPool, user_id: i64, role_id: i64, assigned_by: Option<i64>) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id, assigned_by) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(role_id)
    .bind(assigned_by)
    .execute(pool.inner())
    .await?;

    Ok(())
}

/// Remove a role from a user.
pub async fn remove_role(pool: &DbPool, user_id: i64, role_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = ? AND role_id = ?")
        .bind(user_id)
        .bind(role_id)
        .execute(pool.inner())
        .await?;

    Ok(())
}

/// Find a role by name.
pub async fn find_role_by_name(pool: &DbPool, name: &str) -> Result<Option<Role>> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT id, name, description, created_at, updated_at FROM roles WHERE name = ?"
    )
    .bind(name)
    .fetch_optional(pool.inner())
    .await?;

    Ok(role)
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
    async fn test_find_role_by_name() {
        let pool = create_test_pool().await;

        let role = find_role_by_name(&pool, "admin").await.unwrap();
        assert!(role.is_some());
        assert_eq!(role.unwrap().name, "admin");

        let not_found = find_role_by_name(&pool, "nonexistent").await.unwrap();
        assert!(not_found.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_assign_and_remove_role() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        // Create a user
        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Find admin role
        let admin_role = find_role_by_name(&pool, "admin").await.unwrap().unwrap();

        // Assign role
        assign_role(&pool, user.id, admin_role.id, None).await.unwrap();

        // Check user has role
        let roles = get_user_roles(&pool, user.id).await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin");

        // Remove role
        remove_role(&pool, user.id, admin_role.id).await.unwrap();

        // Check role removed
        let roles = get_user_roles(&pool, user.id).await.unwrap();
        assert_eq!(roles.len(), 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_check_permission() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        // Create a user
        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // User should not have any permissions initially
        let has_perm = check_permission(&pool, user.id, "inventory.read").await.unwrap();
        assert!(!has_perm);

        // Assign admin role
        let admin_role = find_role_by_name(&pool, "admin").await.unwrap().unwrap();
        assign_role(&pool, user.id, admin_role.id, None).await.unwrap();

        // User should now have inventory.read permission (admin has all permissions)
        let has_perm = check_permission(&pool, user.id, "inventory.read").await.unwrap();
        assert!(has_perm);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_permissions() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        // Create a user
        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Assign warehouse_operator role
        let role = find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
        assign_role(&pool, user.id, role.id, None).await.unwrap();

        // Get permissions
        let permissions = get_user_permissions(&pool, user.id).await.unwrap();
        assert!(permissions.len() > 0);

        // Check that warehouse_operator has expected permissions
        let perm_names: Vec<String> = permissions.iter().map(|p| p.name.clone()).collect();
        assert!(perm_names.contains(&"inventory.read".to_string()));
        assert!(perm_names.contains(&"warehouse.read".to_string()));
        assert!(perm_names.contains(&"warehouse.execute".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_require_permission() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        // Create a user
        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        // Should fail without permission
        let result = require_permission(&pool, user.id, "inventory.read").await;
        assert!(result.is_err());

        // Assign role with permission
        let role = find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
        assign_role(&pool, user.id, role.id, None).await.unwrap();

        // Should succeed with permission
        let result = require_permission(&pool, user.id, "inventory.read").await;
        assert!(result.is_ok());

        pool.close().await;
    }
}

