//! User model and repository.

use crate::error::{AuthError, Result};
use async_trait::async_trait;
use persistence::DbPool;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;

/// User model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    /// Hash a password using SHA-256.
    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify a password against the stored hash.
    pub fn verify_password(&self, password: &str) -> bool {
        if let Some(hash) = &self.password_hash {
            let computed_hash = Self::hash_password(password);
            computed_hash == *hash
        } else {
            false
        }
    }
}

/// User repository for database operations.
#[async_trait]
pub trait UserRepository {
    /// Find a user by ID.
    async fn find_by_id(&self, id: i64) -> Result<Option<User>>;

    /// Find a user by username.
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Find a user by email.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Create a new user.
    async fn create(&self, username: &str, email: &str, password: &str) -> Result<User>;

    /// Update user's last login time.
    async fn update_last_login(&self, user_id: i64) -> Result<()>;

    /// Deactivate a user.
    async fn deactivate(&self, user_id: i64) -> Result<()>;

    /// Activate a user.
    async fn activate(&self, user_id: i64) -> Result<()>;
}

/// SQLite implementation of UserRepository.
pub struct SqliteUserRepository {
    pool: DbPool,
}

impl SqliteUserRepository {
    /// Create a new SQLite user repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
             FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
             FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(self.pool.inner())
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
             FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(self.pool.inner())
        .await?;

        Ok(user)
    }

    async fn create(&self, username: &str, email: &str, password: &str) -> Result<User> {
        let password_hash = User::hash_password(password);

        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash, is_active) VALUES (?, ?, ?, 1)"
        )
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .execute(self.pool.inner())
        .await?;

        let user_id = result.last_insert_rowid();

        self.find_by_id(user_id)
            .await?
            .ok_or_else(|| AuthError::Internal("Failed to retrieve created user".to_string()))
    }

    async fn update_last_login(&self, user_id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET last_login_at = datetime('now') WHERE id = ?")
            .bind(user_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }

    async fn deactivate(&self, user_id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = 0, updated_at = datetime('now') WHERE id = ?")
            .bind(user_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }

    async fn activate(&self, user_id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = 1, updated_at = datetime('now') WHERE id = ?")
            .bind(user_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_hash_password() {
        let hash1 = User::hash_password("password123");
        let hash2 = User::hash_password("password123");
        let hash3 = User::hash_password("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_create_user() {
        let pool = create_test_pool().await;
        let repo = SqliteUserRepository::new(pool.clone());

        let user = repo.create("testuser", "test@example.com", "password123").await.unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(user.password_hash.is_some());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_find_by_username() {
        let pool = create_test_pool().await;
        let repo = SqliteUserRepository::new(pool.clone());

        repo.create("testuser", "test@example.com", "password123").await.unwrap();

        let user = repo.find_by_username("testuser").await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "testuser");

        let not_found = repo.find_by_username("nonexistent").await.unwrap();
        assert!(not_found.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_verify_password() {
        let pool = create_test_pool().await;
        let repo = SqliteUserRepository::new(pool.clone());

        let user = repo.create("testuser", "test@example.com", "password123").await.unwrap();

        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_deactivate_activate() {
        let pool = create_test_pool().await;
        let repo = SqliteUserRepository::new(pool.clone());

        let user = repo.create("testuser", "test@example.com", "password123").await.unwrap();
        assert!(user.is_active);

        repo.deactivate(user.id).await.unwrap();
        let user = repo.find_by_id(user.id).await.unwrap().unwrap();
        assert!(!user.is_active);

        repo.activate(user.id).await.unwrap();
        let user = repo.find_by_id(user.id).await.unwrap().unwrap();
        assert!(user.is_active);

        pool.close().await;
    }
}

