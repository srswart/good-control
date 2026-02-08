//! Warehouse service for coordinating receiving workflows with inventory updates.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{PersistenceError, Result};
use crate::inventory_service::InventoryService;
use warehouse::receiving::{ReceivingTask, QCOutcome, QuarantineStatus};

/// Warehouse service for coordinating receiving operations with inventory.
pub struct WarehouseService {
    pool: SqlitePool,
    inventory_service: InventoryService,
}

impl WarehouseService {
    /// Create a new warehouse service.
    pub fn new(pool: SqlitePool) -> Self {
        let inventory_service = InventoryService::new(pool.clone());
        Self {
            pool,
            inventory_service,
        }
    }

    /// Complete receiving task and update inventory ledger.
    ///
    /// This operation:
    /// 1. Validates the receiving task is ready for completion
    /// 2. For each line with QC Pass and not in quarantine (or released from quarantine):
    ///    - Updates inventory positions via inventory service
    /// 3. Records inventory movements for all updated lines
    ///
    /// Lines in quarantine or rejected are skipped (no inventory update).
    pub async fn complete_receiving_task(
        &self,
        task: &ReceivingTask,
        location_id: &str,
        actor: &str,
    ) -> Result<Vec<String>> {
        if task.status != warehouse::receiving::ReceivingTaskStatus::Completed {
            return Err(PersistenceError::Generic(format!(
                "Receiving task must be completed before updating inventory. Current status: {:?}",
                task.status
            )));
        }

        let mut movement_ids = Vec::new();

        for line in &task.lines {
            // Skip lines that are rejected or still in quarantine
            if line.quarantine_status == QuarantineStatus::Rejected {
                continue;
            }

            if line.quarantine_status == QuarantineStatus::Quarantined {
                // Still in quarantine, skip inventory update
                continue;
            }

            // Only update inventory for lines with QC Pass
            let qc_result = match &line.qc_result {
                Some(qc) => qc,
                None => continue, // No QC result, skip
            };

            if qc_result.outcome != QCOutcome::Pass {
                // Only Pass outcomes update inventory
                continue;
            }

            // Update inventory for this line
            // Only count received quantity (not expected, in case of discrepancies)
            if line.received_quantity > 0 {
                let reference = format!("{}:{}", task.task_number, line.id);
                match self
                    .inventory_service
                    .receive_inventory(
                        &line.sku,
                        location_id,
                        line.received_quantity as i64,
                        actor,
                        Some(reference),
                    )
                    .await
                {
                    Ok(movement_id) => movement_ids.push(movement_id),
                    Err(e) => {
                        return Err(PersistenceError::Generic(format!(
                            "Failed to update inventory for line {}: {}",
                            line.id, e
                        )));
                    }
                }
            }
        }

        Ok(movement_ids)
    }

    /// Release line from quarantine and update inventory.
    ///
    /// This operation:
    /// 1. Validates the line is in quarantine
    /// 2. Updates inventory for the released line
    /// 3. Records inventory movement
    pub async fn release_from_quarantine_and_update_inventory(
        &self,
        task: &ReceivingTask,
        line_id: Uuid,
        location_id: &str,
        actor: &str,
    ) -> Result<String> {
        let line = task
            .lines
            .iter()
            .find(|l| l.id == line_id)
            .ok_or_else(|| PersistenceError::NotFound {
                entity_type: "ReceivingLine",
                id: line_id.to_string(),
            })?;

        if line.quarantine_status != QuarantineStatus::Quarantined {
            return Err(PersistenceError::Generic(format!(
                "Line is not in quarantine. Current status: {:?}",
                line.quarantine_status
            )));
        }

        // Require QC Pass before releasing from quarantine
        let qc_result = line.qc_result.as_ref().ok_or_else(|| {
            PersistenceError::Generic("QC result required before releasing from quarantine".to_string())
        })?;

        if qc_result.outcome != QCOutcome::Pass {
            return Err(PersistenceError::Generic(format!(
                "Cannot release from quarantine with QC outcome: {:?}. Must be Pass.",
                qc_result.outcome
            )));
        }

        if line.received_quantity == 0 {
            return Err(PersistenceError::Generic(
                "Cannot update inventory for line with zero received quantity".to_string(),
            ));
        }

        let reference = format!("{}:{}:quarantine_release", task.task_number, line.id);
        self.inventory_service
            .receive_inventory(
                &line.sku,
                location_id,
                line.received_quantity as i64,
                actor,
                Some(reference),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InventoryPositionRecord;
    use procurement::{PurchaseOrder, PurchaseOrderLine};
    use warehouse::receiving::{ReceivingTask, QCOutcome};

    async fn setup_test_service() -> (SqlitePool, WarehouseService) {
        use crate::pool::DbPoolConfig;
        let db_path = std::env::temp_dir()
            .join(format!("warehouse_svc_{}.db", uuid::Uuid::new_v4()));
        let config = DbPoolConfig {
            database_url: format!("sqlite:{}", db_path.to_string_lossy()),
            max_connections: 5,
            connection_timeout_secs: 30,
        };
        let db_pool = crate::pool::DbPool::new(&config).await.unwrap();
        crate::migrations::run_migrations(&db_pool).await.unwrap();
        let pool = db_pool.inner().clone();
        let service = WarehouseService::new(pool.clone());
        (pool, service)
    }

    async fn create_test_item_and_location(pool: &SqlitePool, sku: &str, location_id: &str) {
        // Create item
        sqlx::query(
            r#"
            INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode)
            VALUES (?, ?, ?, ?, 'part', 'none')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(sku)
        .bind(format!("{} Name", sku))
        .bind(format!("{} 名称", sku))
        .execute(pool)
        .await
        .unwrap();

        // Create location with unique bin based on location_id
        let bin_id = format!("BIN-{}", &location_id[..8]);
        sqlx::query(
            r#"
            INSERT INTO locations (id, warehouse, zone, bin)
            VALUES (?, 'WH-001', 'RECEIVING', ?)
            "#,
        )
        .bind(location_id)
        .bind(bin_id)
        .execute(pool)
        .await
        .unwrap();
    }

    fn create_test_receiving_task() -> ReceivingTask {
        let mut po = PurchaseOrder::new("PO-2026-001".to_string(), Uuid::new_v4());
        po.lines.push(PurchaseOrderLine::new("SKU-001".to_string(), 10, 100.0));
        po.lines.push(PurchaseOrderLine::new("SKU-002".to_string(), 5, 50.0));

        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        // Complete first line
        let line_id_1 = task.lines[0].id;
        task.record_receipt(line_id_1, 10).unwrap();
        task.record_qc_result(line_id_1, QCOutcome::Pass, "Inspector A".to_string(), None)
            .unwrap();

        // Complete second line
        let line_id_2 = task.lines[1].id;
        task.record_receipt(line_id_2, 5).unwrap();
        task.record_qc_result(line_id_2, QCOutcome::Pass, "Inspector A".to_string(), None)
            .unwrap();

        task.complete().unwrap();
        task
    }

    #[tokio::test]
    async fn test_complete_receiving_task_updates_inventory() {
        let (pool, service) = setup_test_service().await;
        let location_id_1 = Uuid::new_v4().to_string();
        let location_id_2 = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id_1).await;
        create_test_item_and_location(&pool, "SKU-002", &location_id_2).await;

        let task = create_test_receiving_task();
        // Use location_id_1 for both SKUs (same receiving location)
        let result = service
            .complete_receiving_task(&task, &location_id_1, "operator1")
            .await;

        assert!(result.is_ok());
        let movement_ids = result.unwrap();
        assert_eq!(movement_ids.len(), 2); // Two lines updated

        // Verify inventory positions updated
        let position_1: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("SKU-001")
                .bind(&location_id_1)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position_1.quantity_on_hand, 10);
        assert_eq!(position_1.quantity_available, 10);

        let position_2: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("SKU-002")
                .bind(&location_id_1)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position_2.quantity_on_hand, 5);
        assert_eq!(position_2.quantity_available, 5);
    }

    #[tokio::test]
    async fn test_complete_receiving_task_skips_quarantine_items() {
        let (pool, service) = setup_test_service().await;
        let location_id_1 = Uuid::new_v4().to_string();
        let location_id_2 = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id_1).await;
        create_test_item_and_location(&pool, "SKU-002", &location_id_2).await;

        let mut task = create_test_receiving_task();
        // Put second line in quarantine
        let line_id_2 = task.lines[1].id;
        task.lines[1].quarantine_status = QuarantineStatus::Quarantined;

        let result = service
            .complete_receiving_task(&task, &location_id_1, "operator1")
            .await;

        assert!(result.is_ok());
        let movement_ids = result.unwrap();
        assert_eq!(movement_ids.len(), 1); // Only first line updated

        // Verify only first line updated
        let position_1: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("SKU-001")
                .bind(&location_id_1)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position_1.quantity_on_hand, 10);

        // Second line should not have inventory (still in quarantine)
        let position_2_result: Option<InventoryPositionRecord> =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("SKU-002")
                .bind(&location_id_1)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(position_2_result.is_none());
    }

    #[tokio::test]
    async fn test_complete_receiving_task_skips_rejected_items() {
        let (pool, service) = setup_test_service().await;
        let location_id_1 = Uuid::new_v4().to_string();
        let location_id_2 = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id_1).await;
        create_test_item_and_location(&pool, "SKU-002", &location_id_2).await;

        let mut task = create_test_receiving_task();
        // Reject second line
        task.lines[1].quarantine_status = QuarantineStatus::Rejected;

        let result = service
            .complete_receiving_task(&task, &location_id_1, "operator1")
            .await;

        assert!(result.is_ok());
        let movement_ids = result.unwrap();
        assert_eq!(movement_ids.len(), 1); // Only first line updated
    }

    #[tokio::test]
    async fn test_complete_receiving_task_incomplete_fails() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id).await;

        let mut task = create_test_receiving_task();
        task.status = warehouse::receiving::ReceivingTaskStatus::InProgress; // Not completed

        let result = service
            .complete_receiving_task(&task, &location_id, "operator1")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_release_from_quarantine_updates_inventory() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id).await;

        let mut task = create_test_receiving_task();
        let line_id = task.lines[0].id;
        task.lines[0].quarantine_status = QuarantineStatus::Quarantined;

        let result = service
            .release_from_quarantine_and_update_inventory(&task, line_id, &location_id, "operator1")
            .await;

        assert!(result.is_ok());

        // Verify inventory updated
        let position: InventoryPositionRecord =
            sqlx::query_as("SELECT * FROM inventory_positions WHERE sku = ? AND location_id = ?")
                .bind("SKU-001")
                .bind(&location_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(position.quantity_on_hand, 10);
        assert_eq!(position.quantity_available, 10);
    }

    #[tokio::test]
    async fn test_release_from_quarantine_not_quarantined_fails() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id).await;

        let task = create_test_receiving_task();
        let line_id = task.lines[0].id;

        let result = service
            .release_from_quarantine_and_update_inventory(&task, line_id, &location_id, "operator1")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_release_from_quarantine_no_qc_fails() {
        let (pool, service) = setup_test_service().await;
        let location_id = Uuid::new_v4().to_string();

        create_test_item_and_location(&pool, "SKU-001", &location_id).await;

        let mut task = create_test_receiving_task();
        let line_id = task.lines[0].id;
        task.lines[0].quarantine_status = QuarantineStatus::Quarantined;
        task.lines[0].qc_result = None; // Remove QC result

        let result = service
            .release_from_quarantine_and_update_inventory(&task, line_id, &location_id, "operator1")
            .await;

        assert!(result.is_err());
    }
}
