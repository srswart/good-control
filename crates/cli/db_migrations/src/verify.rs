//! Database integrity verification.

use crate::error::{DbMigrationError, Result};
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// Database verification result.
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    /// Whether the database passed all integrity checks.
    pub passed: bool,
    /// List of issues found (empty if passed).
    pub issues: Vec<String>,
    /// Database file size in bytes.
    pub file_size: u64,
    /// Number of tables in the database.
    pub table_count: usize,
    /// SQLite integrity check result.
    pub integrity_check: String,
}

/// Verify database integrity.
///
/// This performs several checks:
/// - File exists and is readable
/// - SQLite PRAGMA integrity_check
/// - Schema validation
/// - Table count
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A verification result with details about any issues found.
pub async fn verify_database(db_path: &Path) -> Result<VerificationResult> {
    info!("Verifying database: {}", db_path.display());

    let mut issues = Vec::new();

    // Check if file exists
    if !db_path.exists() {
        return Err(DbMigrationError::InvalidPath(format!(
            "Database file does not exist: {}",
            db_path.display()
        )));
    }

    // Get file size
    let metadata = tokio::fs::metadata(db_path).await?;
    let file_size = metadata.len();

    info!("Database file size: {} bytes", file_size);

    // Connect to database
    let pool = connect_to_database(db_path).await?;

    // Run SQLite integrity check
    let integrity_result = check_integrity(&pool).await?;

    if integrity_result != "ok" {
        issues.push(format!("Integrity check failed: {}", integrity_result));
    }

    // Count tables
    let table_count = count_tables(&pool).await?;

    info!("Database has {} tables", table_count);

    // Check for migration table
    let has_migrations = check_migrations_table(&pool).await?;

    if !has_migrations {
        issues.push("Migration tracking table not found".to_string());
    }

    pool.close().await;

    let passed = issues.is_empty();

    if passed {
        info!("Database verification passed");
    } else {
        info!("Database verification found {} issue(s)", issues.len());
    }

    Ok(VerificationResult {
        passed,
        issues,
        file_size,
        table_count,
        integrity_check: integrity_result,
    })
}

/// Connect to the SQLite database.
async fn connect_to_database(db_path: &Path) -> Result<SqlitePool> {
    let db_url = format!("sqlite:{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&db_url)?
        .read_only(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    Ok(pool)
}

/// Run SQLite PRAGMA integrity_check.
async fn check_integrity(pool: &SqlitePool) -> Result<String> {
    let row = sqlx::query("PRAGMA integrity_check")
        .fetch_one(pool)
        .await?;

    let result: String = row.try_get(0)?;

    Ok(result)
}

/// Count the number of tables in the database.
async fn count_tables(pool: &SqlitePool) -> Result<usize> {
    let row = sqlx::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_one(pool)
    .await?;

    let count: i64 = row.try_get(0)?;

    Ok(count as usize)
}

/// Check if the migrations table exists.
async fn check_migrations_table(pool: &SqlitePool) -> Result<bool> {
    let row = sqlx::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'"
    )
    .fetch_one(pool)
    .await?;

    let count: i64 = row.try_get(0)?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_verify_nonexistent_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nonexistent.db");

        let result = verify_database(&db_path).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbMigrationError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn test_verify_invalid_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("invalid.db");

        // Create an invalid database file (not a SQLite database)
        tokio::fs::write(&db_path, b"not a sqlite database").await.unwrap();

        // This should fail because it's not a valid SQLite database
        let result = verify_database(&db_path).await;
        // Invalid file will fail to connect
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_valid_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("valid.db");

        // Create a valid SQLite database
        let db_url = format!("sqlite:{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)
            .unwrap()
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();

        // Create a simple table
        sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();

        pool.close().await;

        // Now verify it
        let result = verify_database(&db_path).await;
        assert!(result.is_ok());

        let verification = result.unwrap();
        assert_eq!(verification.integrity_check, "ok");
        assert!(verification.file_size > 0);
        assert_eq!(verification.table_count, 1);
    }
}

