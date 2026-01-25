//! Database backup functionality.

use crate::error::{DbMigrationError, Result};
use std::path::Path;
use tracing::info;

/// Backup a SQLite database to a file.
///
/// This performs a file-based copy of the SQLite database, which is safe
/// for SQLite databases that are not currently being written to.
///
/// # Arguments
///
/// * `db_path` - Path to the source SQLite database file
/// * `backup_path` - Path where the backup should be saved
///
/// # Returns
///
/// The size of the backup file in bytes.
pub async fn backup_database(db_path: &Path, backup_path: &Path) -> Result<u64> {
    info!(
        "Backing up database from {} to {}",
        db_path.display(),
        backup_path.display()
    );

    // Verify source database exists
    if !db_path.exists() {
        return Err(DbMigrationError::InvalidPath(format!(
            "Source database does not exist: {}",
            db_path.display()
        )));
    }

    // Ensure backup directory exists
    if let Some(parent) = backup_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
            info!("Created backup directory: {}", parent.display());
        }
    }

    // Check if backup file already exists
    if backup_path.exists() {
        return Err(DbMigrationError::Backup(format!(
            "Backup file already exists: {}. Please remove it first or choose a different path.",
            backup_path.display()
        )));
    }

    // Perform the backup using file copy
    // For SQLite, this is safe when the database is not being actively written to
    let bytes_copied = tokio::fs::copy(db_path, backup_path).await?;

    info!(
        "Successfully backed up database ({} bytes)",
        bytes_copied
    );

    // Also backup the WAL and SHM files if they exist (SQLite Write-Ahead Log)
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");

    if wal_path.exists() {
        let backup_wal = backup_path.with_extension("db-wal");
        tokio::fs::copy(&wal_path, &backup_wal).await?;
        info!("Backed up WAL file");
    }

    if shm_path.exists() {
        let backup_shm = backup_path.with_extension("db-shm");
        tokio::fs::copy(&shm_path, &backup_shm).await?;
        info!("Backed up SHM file");
    }

    Ok(bytes_copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("source.db");
        let backup_path = temp_dir.path().join("backup.db");

        // Create a dummy database file
        tokio::fs::write(&db_path, b"test database content").await.unwrap();

        let result = backup_database(&db_path, &backup_path).await;
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert_eq!(bytes, 21); // "test database content" is 21 bytes

        // Verify backup file exists and has correct content
        assert!(backup_path.exists());
        let content = tokio::fs::read(&backup_path).await.unwrap();
        assert_eq!(content, b"test database content");
    }

    #[tokio::test]
    async fn test_backup_nonexistent_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nonexistent.db");
        let backup_path = temp_dir.path().join("backup.db");

        let result = backup_database(&db_path, &backup_path).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbMigrationError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn test_backup_to_existing_file() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("source.db");
        let backup_path = temp_dir.path().join("backup.db");

        // Create source and backup files
        tokio::fs::write(&db_path, b"source").await.unwrap();
        tokio::fs::write(&backup_path, b"existing").await.unwrap();

        let result = backup_database(&db_path, &backup_path).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbMigrationError::Backup(_)));
    }

    #[tokio::test]
    async fn test_backup_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("source.db");
        let backup_path = temp_dir.path().join("backups").join("backup.db");

        // Create source database
        tokio::fs::write(&db_path, b"test").await.unwrap();

        let result = backup_database(&db_path, &backup_path).await;
        assert!(result.is_ok());

        // Verify backup directory was created
        assert!(backup_path.parent().unwrap().exists());
        assert!(backup_path.exists());
    }
}

