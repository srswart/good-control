//! Inventory service for atomic multi-step operations.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{PersistenceError, Result};
use crate::inventory_repository::{
    InventoryMovementRecord, InventoryPositionRecord, InventoryRepository, ItemRecord,
    LocationRecord, SqliteInventoryRepository,
};
use sqlx::Acquire;

/// Inventory service for coordinating multi-step operations.
pub struct InventoryService {
    pool: SqlitePool,
    repository: SqliteInventoryRepository,
}

impl InventoryService {
    /// Create a new inventory service.
    pub fn new(pool: SqlitePool) -> Self {
        let repository = SqliteInventoryRepository::new(pool.clone());
        Self { pool, repository }
    }

    /// Reserve inventory atomically.
    ///
    /// This operation:
    /// 1. Checks available quantity
    /// 2. Updates position to mark quantity as reserved
    /// 3. Records a reservation movement
    ///
    /// All steps are atomic - if any step fails, the entire operation is rolled back.
    pub async fn reserve_inventory(
        &self,
        sku: &str,
        location_id: &str,
        quantity: i64,
        actor: &str,
        reference: Option<String>,
    ) -> Result<String> {
        if quantity <= 0 {
            return Err(PersistenceError::Generic(
                "Reservation quantity must be positive".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;
            // Get or create position for available status
            let position_result = sqlx::query_as::<_, InventoryPositionRecord>(
                r#"
                SELECT * FROM inventory_positions
                WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL
                "#,
            )
            .bind(sku)
            .bind(location_id)
            .fetch_optional(&mut *tx)
            .await?;

            let position = position_result.ok_or_else(|| PersistenceError::NotFound {
                entity_type: "InventoryPosition",
                id: format!("{}@{}:available", sku, location_id),
            })?;

            // Check if sufficient quantity available
            if position.quantity_available < quantity {
                return Err(PersistenceError::Generic(format!(
                    "Insufficient quantity: requested {}, available {}",
                    quantity, position.quantity_available
                )));
            }

            // Update position to reserve quantity
            let new_reserved = position.quantity_reserved + quantity;
            let new_available = position.quantity_on_hand - new_reserved;

            sqlx::query(
                r#"
                UPDATE inventory_positions
                SET quantity_reserved = ?,
                    quantity_available = ?,
                    last_updated = datetime('now')
                WHERE id = ?
                "#,
            )
            .bind(new_reserved)
            .bind(new_available)
            .bind(&position.id)
            .execute(&mut *tx)
            .await?;

            // Record reservation movement
            let movement_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO inventory_movements (
                    id, timestamp, actor, reason, from_location_id, to_location_id,
                    sku, quantity, status_before, status_after, reference
                )
                VALUES (?, datetime('now'), ?, 'reservation', ?, ?, ?, ?, 'available', 'reserved', ?)
                "#,
            )
            .bind(&movement_id)
            .bind(actor)
            .bind(location_id)
            .bind(location_id)
            .bind(sku)
            .bind(quantity)
            .bind(&reference)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(movement_id)
    }

    /// Release a reservation atomically.
    pub async fn release_reservation(
        &self,
        sku: &str,
        location_id: &str,
        quantity: i64,
        actor: &str,
    ) -> Result<String> {
        if quantity <= 0 {
            return Err(PersistenceError::Generic(
                "Release quantity must be positive".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;
            // Get position
            let position_result = sqlx::query_as::<_, InventoryPositionRecord>(
                r#"
                SELECT * FROM inventory_positions
                WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL
                "#,
            )
            .bind(sku)
            .bind(location_id)
            .fetch_optional(&mut *tx)
            .await?;

            let position = position_result.ok_or_else(|| PersistenceError::NotFound {
                entity_type: "InventoryPosition",
                id: format!("{}@{}:available", sku, location_id),
            })?;

            // Check if sufficient quantity reserved
            if position.quantity_reserved < quantity {
                return Err(PersistenceError::Generic(format!(
                    "Insufficient reserved quantity: requested {}, reserved {}",
                    quantity, position.quantity_reserved
                )));
            }

            // Update position to release reservation
            let new_reserved = position.quantity_reserved - quantity;
            let new_available = position.quantity_on_hand - new_reserved;

            sqlx::query(
                r#"
                UPDATE inventory_positions
                SET quantity_reserved = ?,
                    quantity_available = ?,
                    last_updated = datetime('now')
                WHERE id = ?
                "#,
            )
            .bind(new_reserved)
            .bind(new_available)
            .bind(&position.id)
            .execute(&mut *tx)
            .await?;

            // Record release movement
            let movement_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO inventory_movements (
                    id, timestamp, actor, reason, from_location_id, to_location_id,
                    sku, quantity, status_before, status_after
                )
                VALUES (?, datetime('now'), ?, 'reservation_release', ?, ?, ?, ?, 'reserved', 'available')
                "#,
            )
            .bind(&movement_id)
            .bind(actor)
            .bind(location_id)
            .bind(location_id)
            .bind(sku)
            .bind(quantity)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(movement_id)
    }

    /// Ship reserved inventory atomically.
    ///
    /// This operation:
    /// 1. Verifies reserved quantity exists
    /// 2. Decrements both on_hand and reserved quantities
    /// 3. Records a shipment movement
    pub async fn ship_reserved_inventory(
        &self,
        sku: &str,
        location_id: &str,
        quantity: i64,
        actor: &str,
        reference: Option<String>,
    ) -> Result<String> {
        if quantity <= 0 {
            return Err(PersistenceError::Generic(
                "Ship quantity must be positive".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;
            // Get position
            let position_result = sqlx::query_as::<_, InventoryPositionRecord>(
                r#"
                SELECT * FROM inventory_positions
                WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL
                "#,
            )
            .bind(sku)
            .bind(location_id)
            .fetch_optional(&mut *tx)
            .await?;

            let position = position_result.ok_or_else(|| PersistenceError::NotFound {
                entity_type: "InventoryPosition",
                id: format!("{}@{}:available", sku, location_id),
            })?;

            // Check if sufficient quantity reserved
            if position.quantity_reserved < quantity {
                return Err(PersistenceError::Generic(format!(
                    "Insufficient reserved quantity: requested {}, reserved {}",
                    quantity, position.quantity_reserved
                )));
            }

            // Update position to ship reserved inventory
            let new_on_hand = position.quantity_on_hand - quantity;
            let new_reserved = position.quantity_reserved - quantity;
            let new_available = new_on_hand - new_reserved;

            sqlx::query(
                r#"
                UPDATE inventory_positions
                SET quantity_on_hand = ?,
                    quantity_reserved = ?,
                    quantity_available = ?,
                    last_updated = datetime('now')
                WHERE id = ?
                "#,
            )
            .bind(new_on_hand)
            .bind(new_reserved)
            .bind(new_available)
            .bind(&position.id)
            .execute(&mut *tx)
            .await?;

            // Record shipment movement
            let movement_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO inventory_movements (
                    id, timestamp, actor, reason, from_location_id, to_location_id,
                    sku, quantity, status_before, status_after, reference
                )
                VALUES (?, datetime('now'), ?, 'shipment', ?, NULL, ?, ?, 'reserved', 'in_transit', ?)
                "#,
            )
            .bind(&movement_id)
            .bind(actor)
            .bind(location_id)
            .bind(sku)
            .bind(-quantity) // Negative for outbound
            .bind(&reference)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(movement_id)
    }

    /// Reserve and ship in one atomic operation.
    ///
    /// This is a convenience method that combines reserve and ship.
    /// Useful when reservation and shipment happen immediately.
    pub async fn reserve_and_ship(
        &self,
        sku: &str,
        location_id: &str,
        quantity: i64,
        actor: &str,
        reference: Option<String>,
    ) -> Result<(String, String)> {
        if quantity <= 0 {
            return Err(PersistenceError::Generic(
                "Quantity must be positive".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;
            // Get position
            let position_result = sqlx::query_as::<_, InventoryPositionRecord>(
                r#"
                SELECT * FROM inventory_positions
                WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL
                "#,
            )
            .bind(sku)
            .bind(location_id)
            .fetch_optional(&mut *tx)
            .await?;

            let position = position_result.ok_or_else(|| PersistenceError::NotFound {
                entity_type: "InventoryPosition",
                id: format!("{}@{}:available", sku, location_id),
            })?;

            // Check if sufficient quantity available
            if position.quantity_available < quantity {
                return Err(PersistenceError::Generic(format!(
                    "Insufficient quantity: requested {}, available {}",
                    quantity, position.quantity_available
                )));
            }

            // Update position to ship directly (no intermediate reservation state in DB)
            let new_on_hand = position.quantity_on_hand - quantity;
            let new_available = new_on_hand - position.quantity_reserved;

            sqlx::query(
                r#"
                UPDATE inventory_positions
                SET quantity_on_hand = ?,
                    quantity_available = ?,
                    last_updated = datetime('now')
                WHERE id = ?
                "#,
            )
            .bind(new_on_hand)
            .bind(new_available)
            .bind(&position.id)
            .execute(&mut *tx)
            .await?;

            // Record reservation movement
            let reserve_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO inventory_movements (
                    id, timestamp, actor, reason, from_location_id, to_location_id,
                    sku, quantity, status_before, status_after, reference
                )
                VALUES (?, datetime('now'), ?, 'reservation', ?, ?, ?, ?, 'available', 'reserved', ?)
                "#,
            )
            .bind(&reserve_id)
            .bind(actor)
            .bind(location_id)
            .bind(location_id)
            .bind(sku)
            .bind(quantity)
            .bind(&reference)
            .execute(&mut *tx)
            .await?;

            // Record shipment movement
            let ship_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO inventory_movements (
                    id, timestamp, actor, reason, from_location_id, to_location_id,
                    sku, quantity, status_before, status_after, reference
                )
                VALUES (?, datetime('now'), ?, 'shipment', ?, NULL, ?, ?, 'reserved', 'in_transit', ?)
                "#,
            )
            .bind(&ship_id)
            .bind(actor)
            .bind(location_id)
            .bind(sku)
            .bind(-quantity)
            .bind(&reference)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok((reserve_id, ship_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbPool, DbPoolConfig};
    use chrono::Utc;

    async fn setup_test_service() -> (SqlitePool, InventoryService) {
        let db_path = std::env::temp_dir()
            .join(format!("inv_svc_{}.db", uuid::Uuid::new_v4()));
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

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let service = InventoryService::new(pool.clone());

        (pool, service)
    }

    async fn create_test_item_and_location(
        pool: &SqlitePool,
        sku: &str,
        location_id: &str,
    ) -> (ItemRecord, LocationRecord) {
        let item = ItemRecord {
            id: Uuid::new_v4().to_string(),
            sku: sku.to_string(),
            name_en: "Test Item".to_string(),
            name_zh: "测试项".to_string(),
            description_en: None,
            description_zh: None,
            item_type: "part".to_string(),
            tracking_mode: "none".to_string(),
            manufacturer_part_no: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        sqlx::query(
            r#"
            INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&item.id)
        .bind(&item.sku)
        .bind(&item.name_en)
        .bind(&item.name_zh)
        .bind(&item.item_type)
        .bind(&item.tracking_mode)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(pool)
        .await
        .unwrap();

        let location = LocationRecord {
            id: location_id.to_string(),
            warehouse: "WH01".to_string(),
            zone: "A".to_string(),
            bin: "001".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        sqlx::query(
            r#"
            INSERT INTO locations (id, warehouse, zone, bin, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&location.id)
        .bind(&location.warehouse)
        .bind(&location.zone)
        .bind(&location.bin)
        .bind(&location.created_at)
        .execute(pool)
        .await
        .unwrap();

        (item, location)
    }

    async fn create_initial_inventory(
        pool: &SqlitePool,
        sku: &str,
        location_id: &str,
        quantity: i64,
    ) {
        let position_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO inventory_positions (
                id, sku, location_id, status, quantity_on_hand, quantity_reserved, quantity_available, last_updated
            )
            VALUES (?, ?, ?, 'available', ?, 0, ?, datetime('now'))
            "#,
        )
        .bind(&position_id)
        .bind(sku)
        .bind(location_id)
        .bind(quantity)
        .bind(quantity)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reserve_inventory_success() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-001", &location_id).await;
        create_initial_inventory(&pool, "TEST-001", &location_id, 100).await;

        let result = service
            .reserve_inventory("TEST-001", &location_id, 20, "user1", None)
            .await;

        assert!(result.is_ok());

        // Verify position updated
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("TEST-001")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_on_hand, 100);
        assert_eq!(position.quantity_reserved, 20);
        assert_eq!(position.quantity_available, 80);
    }

    #[tokio::test]
    async fn test_reserve_insufficient_quantity() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-002", &location_id).await;
        create_initial_inventory(&pool, "TEST-002", &location_id, 10).await;

        let result = service
            .reserve_inventory("TEST-002", &location_id, 20, "user1", None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient"));
    }

    #[tokio::test]
    async fn test_release_reservation() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-003", &location_id).await;
        create_initial_inventory(&pool, "TEST-003", &location_id, 100).await;

        // Reserve first
        service
            .reserve_inventory("TEST-003", &location_id, 30, "user1", None)
            .await
            .unwrap();

        // Release
        let result = service
            .release_reservation("TEST-003", &location_id, 30, "user1")
            .await;

        assert!(result.is_ok());

        // Verify position restored
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("TEST-003")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_reserved, 0);
        assert_eq!(position.quantity_available, 100);
    }

    #[tokio::test]
    async fn test_ship_reserved_inventory() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-004", &location_id).await;
        create_initial_inventory(&pool, "TEST-004", &location_id, 100).await;

        // Reserve first
        service
            .reserve_inventory("TEST-004", &location_id, 40, "user1", None)
            .await
            .unwrap();

        // Ship
        let result = service
            .ship_reserved_inventory("TEST-004", &location_id, 40, "user1", None)
            .await;

        assert!(result.is_ok());

        // Verify position updated
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("TEST-004")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_on_hand, 60);
        assert_eq!(position.quantity_reserved, 0);
        assert_eq!(position.quantity_available, 60);
    }

    #[tokio::test]
    async fn test_reserve_and_ship_atomic() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-005", &location_id).await;
        create_initial_inventory(&pool, "TEST-005", &location_id, 100).await;

        let result = service
            .reserve_and_ship("TEST-005", &location_id, 50, "user1", None)
            .await;

        assert!(result.is_ok());
        let (reserve_id, ship_id) = result.unwrap();
        assert!(!reserve_id.is_empty());
        assert!(!ship_id.is_empty());

        // Verify position updated
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("TEST-005")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_on_hand, 50);
        assert_eq!(position.quantity_available, 50);

        // Verify both movements recorded
        let movements: Vec<InventoryMovementRecord> =
            sqlx::query_as("SELECT * FROM inventory_movements WHERE sku = ? ORDER BY timestamp")
                .bind("TEST-005")
                .fetch_all(&pool)
                .await
                .unwrap();

        assert_eq!(movements.len(), 2);
        assert_eq!(movements[0].reason, "reservation");
        assert_eq!(movements[1].reason, "shipment");
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_reservations() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "TEST-006", &location_id).await;
        create_initial_inventory(&pool, "TEST-006", &location_id, 100).await;

        // Spawn 10 concurrent reservation attempts for 10 units each
        let mut handles = vec![];
        for i in 0..10 {
            let service_clone = InventoryService::new(pool.clone());
            let location_clone = location_id.clone();
            let handle = tokio::spawn(async move {
                service_clone
                    .reserve_inventory("TEST-006", &location_clone, 10, &format!("user{}", i), None)
                    .await
            });
            handles.push(handle);
        }

        // Collect results
        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success_count += 1;
            }
        }

        // All 10 should succeed (10 * 10 = 100 total)
        assert_eq!(success_count, 10);

        // Verify final state
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("TEST-006")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_on_hand, 100);
        assert_eq!(position.quantity_reserved, 100);
        assert_eq!(position.quantity_available, 0);
    }
}
