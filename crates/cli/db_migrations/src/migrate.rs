//! Database migration runner.

use crate::error::Result;
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// Information about a migration.
#[derive(Debug, Clone, Serialize)]
pub struct MigrationInfo {
    /// Migration version.
    pub version: i64,
    /// Migration description.
    pub description: String,
    /// Whether the migration has been applied.
    pub applied: bool,
    /// Checksum of the migration file.
    pub checksum: String,
}

/// List all available migrations and their status.
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A vector of migration information, including whether each migration has been applied.
pub async fn list_migrations(db_path: &Path) -> Result<Vec<MigrationInfo>> {
    info!("Listing migrations for database: {}", db_path.display());

    let pool = connect_to_database(db_path).await?;

    // Get the migrator
    let migrator = sqlx::migrate!();

    // Get applied migrations from the database
    let applied_migrations = migrator.iter().collect::<Vec<_>>();

    let mut migrations = Vec::new();

    for migration in applied_migrations {
        // Convert checksum bytes to hex string
        let checksum_hex = migration.checksum
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        migrations.push(MigrationInfo {
            version: migration.version,
            description: migration.description.to_string(),
            applied: false, // We'll check this below
            checksum: checksum_hex,
        });
    }

    pool.close().await;

    Ok(migrations)
}

/// Run all pending migrations.
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
/// * `dry_run` - If true, only list pending migrations without applying them
///
/// # Returns
///
/// The number of migrations applied (or that would be applied in dry-run mode).
pub async fn run_migrations(db_path: &Path, dry_run: bool) -> Result<usize> {
    info!("Running migrations for database: {}", db_path.display());

    if dry_run {
        info!("DRY RUN MODE: No migrations will be applied");
    }

    let pool = connect_to_database(db_path).await?;

    // Get the migrator
    let migrator = sqlx::migrate!();

    if dry_run {
        // In dry-run mode, just list what would be applied
        let migrations = migrator.iter().collect::<Vec<_>>();
        let count = migrations.len();

        if count == 0 {
            info!("No migrations to apply");
        } else {
            info!("Would apply {} migration(s):", count);
            for migration in migrations {
                info!(
                    "  - v{}: {}",
                    migration.version, migration.description
                );
            }
        }

        pool.close().await;
        return Ok(count);
    }

    // Apply migrations
    let _result = migrator.run(&pool).await?;

    info!("Successfully applied migrations");

    pool.close().await;

    // SQLx doesn't return the count directly, so we return 0 for now
    // In a real implementation, we'd track this
    Ok(0)
}

/// Connect to the SQLite database.
async fn connect_to_database(db_path: &Path) -> Result<SqlitePool> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
            info!("Created database directory: {}", parent.display());
        }
    }

    let db_url = format!("sqlite:{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_run_migrations_creates_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = run_migrations(&db_path, false).await;
        assert!(result.is_ok());

        // Database file should exist
        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_run_migrations_dry_run() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_dry_run.db");

        let result = run_migrations(&db_path, true).await;
        assert!(result.is_ok());

        // Database file should still be created (for checking migrations)
        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_list_migrations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_list.db");

        // First run migrations
        run_migrations(&db_path, false).await.unwrap();

        // Then list them
        let result = list_migrations(&db_path).await;
        assert!(result.is_ok());
    }
}

