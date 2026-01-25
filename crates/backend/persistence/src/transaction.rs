//! Transaction helper and tests for commit, rollback, and concurrency behavior.

use crate::error::Result;

/// Begin a SQLite transaction.
pub async fn begin_transaction(pool: &sqlx::SqlitePool) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
    Ok(pool.begin().await?)
}

#[cfg(test)]
mod tests {
    use crate::{DbPool, DbPoolConfig};

    async fn setup_test_pool() -> sqlx::SqlitePool {
        let db_path = std::env::temp_dir()
            .join(format!("inv_tx_{}.db", uuid::Uuid::new_v4()));
        let config = DbPoolConfig {
            database_url: format!("sqlite:{}", db_path.to_string_lossy()),
            max_connections: 5,
            connection_timeout_secs: 30,
        };
        let db_pool = DbPool::new(&config).await.unwrap();
        let pool = db_pool.inner().clone();
        // Improve concurrency characteristics for tests
        sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.unwrap();
        sqlx::query("PRAGMA busy_timeout = 5000").execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let pool = setup_test_pool().await;

        // Create a test table
        sqlx::query("CREATE TABLE test_commit (id INTEGER PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        // Transaction that should commit
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("INSERT INTO test_commit (id, value) VALUES (1, 'test')")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Verify data was committed
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_commit")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_error() {
        let pool = setup_test_pool().await;

        // Create a test table
        sqlx::query("CREATE TABLE test_rollback (id INTEGER PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        // Transaction that should rollback
        {
            let mut tx = pool.begin().await.unwrap();
            sqlx::query("INSERT INTO test_rollback (id, value) VALUES (1, 'test')")
                .execute(&mut *tx)
                .await
                .unwrap();
            // Drop without commit -> rollback
        }

        // Verify data was rolled back
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_rollback")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_explicit_transaction_commit() {
        let pool = setup_test_pool().await;

        // Create a test table
        sqlx::query("CREATE TABLE test_explicit (id INTEGER PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        // Transaction with explicit commit
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("INSERT INTO test_explicit (id, value) VALUES (1, 'test')")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Verify data was committed
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_explicit")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_transactions() {
        let pool = setup_test_pool().await;

        // Create a test table with a counter
        sqlx::query("CREATE TABLE test_concurrent (id INTEGER PRIMARY KEY, counter INTEGER)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO test_concurrent (id, counter) VALUES (1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        // Spawn multiple concurrent transactions that increment the counter
        let mut handles = vec![];

        for _ in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let mut tx = pool_clone.begin().await.unwrap();
                // Read current value
                let current: i64 = sqlx::query_scalar(
                    "SELECT counter FROM test_concurrent WHERE id = 1",
                )
                .fetch_one(&mut *tx)
                .await
                .unwrap();

                // Increment
                sqlx::query("UPDATE test_concurrent SET counter = ? WHERE id = 1")
                    .bind(current + 1)
                    .execute(&mut *tx)
                    .await
                    .unwrap();

                tx.commit().await.unwrap();
            });

            handles.push(handle);
        }

        // Wait for all transactions to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify final counter value
        let final_count: i64 =
            sqlx::query_scalar("SELECT counter FROM test_concurrent WHERE id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(final_count, 10);
    }
}
