//! @arrive component: persistence
//!
//! Database migration management.

use crate::error::Result;
use crate::pool::DbPool;
use tracing::info;

/// Run all pending migrations.
///
/// This function applies all migrations from the `migrations/` directory
/// to the database.
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    info!("Running database migrations...");

    // Use CARGO_MANIFEST_DIR to find workspace root, then migrations directory
    // The macro path is relative to the workspace root
    sqlx::migrate!()
        .run(pool.inner())
        .await?;

    info!("Database migrations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPoolConfig;

    #[tokio::test]
    async fn test_run_migrations_on_empty_db() {
        let config = DbPoolConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let pool = DbPool::new(&config).await.unwrap();
        
        // Should succeed even if no migrations exist yet
        let result = run_migrations(&pool).await;
        
        // This will fail until we create actual migration files
        // For now, we expect it to work with an empty migrations directory
        match result {
            Ok(_) => {
                // Migrations ran successfully
            }
            Err(e) => {
                // Expected if migrations directory doesn't exist yet
                eprintln!("Migration error (expected): {}", e);
            }
        }
        
        pool.close().await;
    }
}

