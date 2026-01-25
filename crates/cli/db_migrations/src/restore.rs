//! Database restore functionality.

use crate::error::{DbMigrationError, Result};
use std::path::Path;
use tracing::{info, warn};

/// Restore a SQLite database from a backup file.
///
/// This performs a file-based copy from the backup to the target database location.
/// The target database will be overwritten if it exists.
///
/// # Arguments
///
/// * `backup_path` - Path to the backup file
/// * `db_path` - Path where the database should be restored
///
/// # Returns
///
/// The size of the restored database in bytes.
pub async fn restore_database(backup_path: &Path, db_path: &Path) -> Result<u64> {
    info!(
        "Restoring database from {} to {}",
        backup_path.display(),
        db_path.display()
    );

    // Verify backup file exists
    if !backup_path.exists() {
        return Err(DbMigrationError::InvalidPath(format!(
            "Backup file does not exist: {}",
            backup_path.display()
        )));
    }

    // Ensure target directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
            info!("Created database directory: {}", parent.display());
        }
    }

    // Warn if target database exists
    if db_path.exists() {
        warn!(
            "Target database exists and will be overwritten: {}",
            db_path.display()
        );

        // Remove existing database and associated files
        tokio::fs::remove_file(db_path).await?;

        // Also remove WAL and SHM files if they exist
        let wal_path = db_path.with_extension("db-wal");
        let shm_path = db_path.with_extension("db-shm");

        if wal_path.exists() {
            tokio::fs::remove_file(&wal_path).await?;
            info!("Removed existing WAL file");
        }

        if shm_path.exists() {
            tokio::fs::remove_file(&shm_path).await?;
            info!("Removed existing SHM file");
        }
    }

    // Perform the restore using file copy
    let bytes_copied = tokio::fs::copy(backup_path, db_path).await?;

    info!(
        "Successfully restored database ({} bytes)",
        bytes_copied
    );

    // Also restore the WAL and SHM files if they exist in the backup
    let backup_wal = backup_path.with_extension("db-wal");
    let backup_shm = backup_path.with_extension("db-shm");

    if backup_wal.exists() {
        let wal_path = db_path.with_extension("db-wal");
        tokio::fs::copy(&backup_wal, &wal_path).await?;
        info!("Restored WAL file");
    }

    if backup_shm.exists() {
        let shm_path = db_path.with_extension("db-shm");
        tokio::fs::copy(&backup_shm, &shm_path).await?;
        info!("Restored SHM file");
    }

    Ok(bytes_copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_restore_database() {
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");
        let db_path = temp_dir.path().join("restored.db");

        // Create a backup file
        tokio::fs::write(&backup_path, b"backup content").await.unwrap();

        let result = restore_database(&backup_path, &db_path).await;
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert_eq!(bytes, 14); // "backup content" is 14 bytes

        // Verify restored file exists and has correct content
        assert!(db_path.exists());
        let content = tokio::fs::read(&db_path).await.unwrap();
        assert_eq!(content, b"backup content");
    }

    #[tokio::test]
    async fn test_restore_nonexistent_backup() {
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("nonexistent.db");
        let db_path = temp_dir.path().join("restored.db");

        let result = restore_database(&backup_path, &db_path).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbMigrationError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn test_restore_overwrites_existing() {
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");
        let db_path = temp_dir.path().join("existing.db");

        // Create backup and existing database
        tokio::fs::write(&backup_path, b"new content").await.unwrap();
        tokio::fs::write(&db_path, b"old content").await.unwrap();

        let result = restore_database(&backup_path, &db_path).await;
        assert!(result.is_ok());

        // Verify existing file was overwritten
        let content = tokio::fs::read(&db_path).await.unwrap();
        assert_eq!(content, b"new content");
    }

    #[tokio::test]
    async fn test_restore_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");
        let db_path = temp_dir.path().join("databases").join("restored.db");

        // Create backup file
        tokio::fs::write(&backup_path, b"test").await.unwrap();

        let result = restore_database(&backup_path, &db_path).await;
        assert!(result.is_ok());

        // Verify database directory was created
        assert!(db_path.parent().unwrap().exists());
        assert!(db_path.exists());
    }
}

