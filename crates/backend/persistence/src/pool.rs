//! @arrive component: persistence
//!
//! Database connection pool management.

use crate::error::{PersistenceError, Result};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::env;
use std::str::FromStr;
use std::time::Duration;

/// Configuration for the database pool.
#[derive(Debug, Clone, Deserialize)]
pub struct DbPoolConfig {
    /// Database URL (e.g., "sqlite:data/drone-inventory.db")
    #[serde(default = "default_database_url")]
    pub database_url: String,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
}

fn default_database_url() -> String {
    "sqlite:data/drone-inventory.db".to_string()
}

fn default_max_connections() -> u32 {
    5
}

fn default_connection_timeout_secs() -> u64 {
    30
}

impl Default for DbPoolConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
            max_connections: default_max_connections(),
            connection_timeout_secs: default_connection_timeout_secs(),
        }
    }
}

impl DbPoolConfig {
    /// Load configuration from environment variables.
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        if let Ok(value) = env::var("APP__DATABASE__URL") {
            config.database_url = value;
        }

        if let Ok(value) = env::var("APP__DATABASE__MAX_CONNECTIONS") {
            config.max_connections = value.parse().map_err(|_| {
                PersistenceError::Configuration(
                    "APP__DATABASE__MAX_CONNECTIONS must be a u32".to_string(),
                )
            })?;
        }

        if let Ok(value) = env::var("APP__DATABASE__CONNECTION_TIMEOUT_SECS") {
            config.connection_timeout_secs = value.parse().map_err(|_| {
                PersistenceError::Configuration(
                    "APP__DATABASE__CONNECTION_TIMEOUT_SECS must be a u64".to_string(),
                )
            })?;
        }

        Ok(config)
    }
}

/// Database connection pool wrapper.
#[derive(Debug, Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    /// Create a new database pool with the given configuration.
    pub async fn new(config: &DbPoolConfig) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(&config.database_url)
            .map_err(|e| PersistenceError::Configuration(e.to_string()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout_secs))
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying SQLx pool.
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn test_default_config() {
        let config = DbPoolConfig::default();
        assert_eq!(config.database_url, "sqlite:data/drone-inventory.db");
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.connection_timeout_secs, 30);
    }

    #[test]
    fn test_load_config_from_env() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        unsafe {
            env::set_var("APP__DATABASE__URL", "sqlite:/data/inventory.db");
            env::set_var("APP__DATABASE__MAX_CONNECTIONS", "10");
            env::set_var("APP__DATABASE__CONNECTION_TIMEOUT_SECS", "45");
        }

        let config = DbPoolConfig::load().unwrap();

        assert_eq!(config.database_url, "sqlite:/data/inventory.db");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout_secs, 45);

        unsafe {
            env::remove_var("APP__DATABASE__URL");
            env::remove_var("APP__DATABASE__MAX_CONNECTIONS");
            env::remove_var("APP__DATABASE__CONNECTION_TIMEOUT_SECS");
        }
    }

    #[test]
    fn test_load_config_rejects_invalid_numbers() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        unsafe {
            env::set_var("APP__DATABASE__MAX_CONNECTIONS", "not-a-number");
        }

        let result = DbPoolConfig::load();
        assert!(result.is_err());

        unsafe {
            env::remove_var("APP__DATABASE__MAX_CONNECTIONS");
        }
    }

    #[tokio::test]
    async fn test_create_pool_with_in_memory_db() {
        let config = DbPoolConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let pool = DbPool::new(&config).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_pool_with_file_db() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DbPoolConfig {
            database_url: format!("sqlite:{}", db_path.display()),
            max_connections: 2,
            connection_timeout_secs: 10,
        };

        let pool = DbPool::new(&config).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        pool.close().await;
    }

    #[tokio::test]
    async fn test_invalid_database_url() {
        let config = DbPoolConfig {
            database_url: "invalid://url".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let result = DbPool::new(&config).await;
        assert!(result.is_err());
        // The error could be either Configuration or Connection depending on the URL format
        match result.unwrap_err() {
            PersistenceError::Configuration(_) | PersistenceError::Connection(_) => {
                // Expected error types
            }
            _ => panic!("Expected Configuration or Connection error"),
        }
    }
}

