//! Admin bootstrap utilities.

use crate::error::Result;
use crate::rbac::{assign_role, find_role_by_name};
use crate::user::{SqliteUserRepository, UserRepository};
use persistence::DbPool;

/// Seed an initial admin user from environment variables if none exists.
pub async fn seed_admin_user(pool: &DbPool) -> Result<()> {
    let admin_email = match std::env::var("APP__ADMIN_EMAIL") {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };
    let admin_username = std::env::var("APP__ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let admin_password = match std::env::var("APP__ADMIN_PASSWORD") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };

    let repo = SqliteUserRepository::new(pool.clone());

    if repo.find_by_email(&admin_email).await?.is_some() {
        return Ok(());
    }

    let user = repo.create(&admin_username, &admin_email, &admin_password).await?;

    if let Some(role) = find_role_by_name(pool, "admin").await? {
        let _ = assign_role(pool, user.id, role.id, None).await;
    }

    Ok(())
}
