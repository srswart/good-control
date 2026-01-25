//! @arrive component: persistence
//!
//! Repository trait pattern for data access.

use crate::error::Result;
use async_trait::async_trait;

/// Generic repository trait for CRUD operations.
///
/// This trait defines the basic operations that all repositories should support.
/// Domain-specific repositories can extend this trait with additional methods.
#[async_trait]
pub trait Repository<T, ID> {
    /// Find an entity by its ID.
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;

    /// Find all entities.
    async fn find_all(&self) -> Result<Vec<T>>;

    /// Save an entity (insert or update).
    async fn save(&self, entity: &T) -> Result<T>;

    /// Delete an entity by its ID.
    async fn delete(&self, id: ID) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::{DbPool, DbPoolConfig};

    // Example entity for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: i64,
        name: String,
    }

    // Example repository implementation
    struct TestRepository {
        _pool: DbPool,
    }

    #[async_trait]
    impl Repository<TestEntity, i64> for TestRepository {
        async fn find_by_id(&self, _id: i64) -> Result<Option<TestEntity>> {
            // Placeholder implementation
            Ok(None)
        }

        async fn find_all(&self) -> Result<Vec<TestEntity>> {
            // Placeholder implementation
            Ok(vec![])
        }

        async fn save(&self, entity: &TestEntity) -> Result<TestEntity> {
            // Placeholder implementation
            Ok(entity.clone())
        }

        async fn delete(&self, _id: i64) -> Result<bool> {
            // Placeholder implementation
            Ok(false)
        }
    }

    #[tokio::test]
    async fn test_repository_trait_compiles() {
        let config = DbPoolConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let pool = DbPool::new(&config).await.unwrap();
        let repo = TestRepository { _pool: pool.clone() };

        // Test that the trait methods can be called
        let result = repo.find_by_id(1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        let all = repo.find_all().await;
        assert!(all.is_ok());
        assert_eq!(all.unwrap().len(), 0);

        pool.close().await;
    }
}

