//! Session and token management.

use crate::error::{AuthError, Result};
use crate::user::{User, UserRepository};
use persistence::DbPool;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use std::time::{SystemTime, UNIX_EPOCH};

/// Session model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
    pub last_accessed_at: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Session manager for creating and validating sessions.
pub struct SessionManager {
    pool: DbPool,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Create a new session for a user.
    pub async fn create_session(
        &self,
        user_id: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session> {
        let token = Self::generate_token(user_id);
        let expires_at = Self::calculate_expiry(24 * 60 * 60); // 24 hours

        sqlx::query(
            "INSERT INTO sessions (user_id, token, expires_at, ip_address, user_agent) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(&token)
        .bind(&expires_at)
        .bind(&ip_address)
        .bind(&user_agent)
        .execute(self.pool.inner())
        .await?;

        let session = self.find_by_token(&token).await?
            .ok_or_else(|| AuthError::Internal("Failed to retrieve created session".to_string()))?;

        Ok(session)
    }

    /// Validate a session token and return the session if valid.
    pub async fn validate_session(&self, token: &str) -> Result<Session> {
        let session = self.find_by_token(token).await?
            .ok_or(AuthError::InvalidToken)?;

        // Check if session is expired
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if session.expires_at < now {
            // Delete expired session
            self.delete_session(session.id).await?;
            return Err(AuthError::TokenExpired);
        }

        // Update last accessed time
        self.update_last_accessed(session.id).await?;

        Ok(session)
    }

    /// Find a session by token.
    async fn find_by_token(&self, token: &str) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, token, expires_at, created_at, last_accessed_at, ip_address, user_agent
             FROM sessions WHERE token = ?"
        )
        .bind(token)
        .fetch_optional(self.pool.inner())
        .await?;

        Ok(session)
    }

    /// Update the last accessed time for a session.
    async fn update_last_accessed(&self, session_id: i64) -> Result<()> {
        sqlx::query("UPDATE sessions SET last_accessed_at = datetime('now') WHERE id = ?")
            .bind(session_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }

    /// Delete a session.
    pub async fn delete_session(&self, session_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }

    /// Delete all sessions for a user.
    pub async fn delete_user_sessions(&self, user_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool.inner())
            .await?;

        Ok(())
    }

    /// Clean up expired sessions.
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < datetime('now')")
            .execute(self.pool.inner())
            .await?;

        Ok(result.rows_affected())
    }

    /// Generate a secure token.
    fn generate_token(user_id: i64) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let data = format!("{}-{}-{}", user_id, timestamp, rand::random::<u64>());
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Calculate expiry time in ISO 8601 format.
    fn calculate_expiry(seconds: i64) -> String {
        let expiry = chrono::Utc::now() + chrono::Duration::seconds(seconds);
        expiry.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

/// Create a session for a user (convenience function).
pub async fn create_session(
    pool: &DbPool,
    user_id: i64,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<Session> {
    let manager = SessionManager::new(pool.clone());
    manager.create_session(user_id, ip_address, user_agent).await
}

/// Validate a session token (convenience function).
pub async fn validate_session(pool: &DbPool, token: &str) -> Result<Session> {
    let manager = SessionManager::new(pool.clone());
    manager.validate_session(token).await
}

/// Authenticate a user with username and password, returning a session.
pub async fn authenticate<R: UserRepository>(
    repo: &R,
    pool: &DbPool,
    username: &str,
    password: &str,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<(User, Session)> {
    // Find user
    let user = repo.find_by_username(username).await?
        .ok_or(AuthError::InvalidCredentials)?;

    // Check if user is active
    if !user.is_active {
        return Err(AuthError::InvalidCredentials);
    }

    // Verify password
    if !user.verify_password(password) {
        return Err(AuthError::InvalidCredentials);
    }

    // Update last login
    repo.update_last_login(user.id).await?;

    // Create session
    let session = create_session(pool, user.id, ip_address, user_agent).await?;

    Ok((user, session))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::SqliteUserRepository;
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
    async fn test_create_session() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        let manager = SessionManager::new(pool.clone());
        let session = manager.create_session(user.id, None, None).await.unwrap();

        assert_eq!(session.user_id, user.id);
        assert!(!session.token.is_empty());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_validate_session() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        let manager = SessionManager::new(pool.clone());
        let session = manager.create_session(user.id, None, None).await.unwrap();

        // Validate session
        let validated = manager.validate_session(&session.token).await.unwrap();
        assert_eq!(validated.id, session.id);

        // Invalid token should fail
        let result = manager.validate_session("invalid_token").await;
        assert!(result.is_err());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_authenticate() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        user_repo.create("testuser", "test@example.com", "password123").await.unwrap();

        // Successful authentication
        let result = authenticate(&user_repo, &pool, "testuser", "password123", None, None).await;
        assert!(result.is_ok());

        let (user, session) = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert!(!session.token.is_empty());

        // Failed authentication (wrong password)
        let result = authenticate(&user_repo, &pool, "testuser", "wrongpassword", None, None).await;
        assert!(result.is_err());

        // Failed authentication (nonexistent user)
        let result = authenticate(&user_repo, &pool, "nonexistent", "password", None, None).await;
        assert!(result.is_err());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_session() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();

        let manager = SessionManager::new(pool.clone());
        let session = manager.create_session(user.id, None, None).await.unwrap();

        // Delete session
        manager.delete_session(session.id).await.unwrap();

        // Session should no longer be valid
        let result = manager.validate_session(&session.token).await;
        assert!(result.is_err());

        pool.close().await;
    }
}

