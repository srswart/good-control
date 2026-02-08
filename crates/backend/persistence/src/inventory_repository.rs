//! @arrive component: persistence
//!
//! Inventory repository records and trait definitions.

use crate::error::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

/// Item record (items table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemRecord {
    pub id: String,
    pub sku: String,
    pub name_en: String,
    pub name_zh: String,
    pub description_en: Option<String>,
    pub description_zh: Option<String>,
    pub item_type: String,
    pub tracking_mode: String,
    pub manufacturer_part_no: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Location record (locations table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocationRecord {
    pub id: String,
    pub warehouse: String,
    pub zone: String,
    pub bin: String,
    pub created_at: String,
}

/// Inventory position record (inventory_positions table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryPositionRecord {
    pub id: String,
    pub sku: String,
    pub location_id: String,
    pub status: String,
    pub lot_id: Option<String>,
    pub quantity_on_hand: i64,
    pub quantity_reserved: i64,
    pub quantity_available: i64,
    pub last_updated: String,
}

/// Inventory movement record (inventory_movements table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryMovementRecord {
    pub id: String,
    pub timestamp: String,
    pub actor: String,
    pub reason: String,
    pub from_location_id: Option<String>,
    pub to_location_id: Option<String>,
    pub sku: String,
    pub quantity: i64,
    pub serial_numbers: Option<String>,
    pub lot_id: Option<String>,
    pub status_before: String,
    pub status_after: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

/// Inventory repository trait (placeholder for future specialization)
#[async_trait]
pub trait InventoryRepository: Send + Sync {}

/// SQLite implementation placeholder
pub struct SqliteInventoryRepository {
    _pool: SqlitePool,
}

impl SqliteInventoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}

/// Get current inventory position by SKU, location, and status.
pub async fn get_current_position(
    pool: &SqlitePool,
    sku: &str,
    location_id: &str,
    status: &str,
) -> Result<Option<InventoryPositionRecord>> {
    let rec = sqlx::query_as::<_, InventoryPositionRecord>(
        r#"
        SELECT * FROM inventory_positions
        WHERE sku = ? AND location_id = ? AND status = ? AND lot_id IS NULL
        "#,
    )
    .bind(sku)
    .bind(location_id)
    .bind(status)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// Build a LIKE pattern for matching a value inside a JSON array stored as text.
fn json_array_like_pattern(value: &str) -> String {
    // Simple match for \"value\" inside the JSON array string
    format!("%\"{}\"%", value.replace('"', "\\\""))
}

/// Get full movement genealogy for a serial number (ordered by time asc).
pub async fn get_serial_genealogy(
    pool: &SqlitePool,
    serial_number: &str,
) -> Result<Vec<InventoryMovementRecord>> {
    let pattern = json_array_like_pattern(serial_number);
    let rows = sqlx::query_as::<_, InventoryMovementRecord>(
        r#"
        SELECT * FROM inventory_movements
        WHERE serial_numbers IS NOT NULL
          AND serial_numbers LIKE ?
        ORDER BY timestamp ASC
        "#,
    )
    .bind(pattern)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get shipment movements for a serial number (where-shipped).
pub async fn get_serial_shipments(
    pool: &SqlitePool,
    serial_number: &str,
) -> Result<Vec<InventoryMovementRecord>> {
    let pattern = json_array_like_pattern(serial_number);
    let rows = sqlx::query_as::<_, InventoryMovementRecord>(
        r#"
        SELECT * FROM inventory_movements
        WHERE reason = 'shipment'
          AND serial_numbers IS NOT NULL
          AND serial_numbers LIKE ?
        ORDER BY timestamp ASC
        "#,
    )
    .bind(pattern)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get full movement genealogy for a lot number (ordered by time asc).
pub async fn get_lot_genealogy_by_number(
    pool: &SqlitePool,
    lot_number: &str,
) -> Result<Vec<InventoryMovementRecord>> {
    let lot_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM lots WHERE lot_number = ?",
    )
    .bind(lot_number)
    .fetch_optional(pool)
    .await?;

    if let Some(id) = lot_id {
        let rows = sqlx::query_as::<_, InventoryMovementRecord>(
            r#"
            SELECT * FROM inventory_movements
            WHERE lot_id = ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    } else {
        Ok(vec![])
    }
}

/// Get shipment movements for a lot number (where-shipped).
pub async fn get_lot_shipments_by_number(
    pool: &SqlitePool,
    lot_number: &str,
) -> Result<Vec<InventoryMovementRecord>> {
    let lot_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM lots WHERE lot_number = ?",
    )
    .bind(lot_number)
    .fetch_optional(pool)
    .await?;

    if let Some(id) = lot_id {
        let rows = sqlx::query_as::<_, InventoryMovementRecord>(
            r#"
            SELECT * FROM inventory_movements
            WHERE reason = 'shipment' AND lot_id = ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    } else {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbPool, DbPoolConfig};
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_pool() -> SqlitePool {
        let db_path = std::env::temp_dir().join(format!("inv_repo_{}.db", Uuid::new_v4()));
        let config = DbPoolConfig {
            database_url: format!("sqlite:{}", db_path.to_string_lossy()),
            max_connections: 5,
            connection_timeout_secs: 30,
        };
        let db_pool = DbPool::new(&config).await.unwrap();
        let pool = db_pool.inner().clone();
        // Tune for tests
        sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.unwrap();
        sqlx::query("PRAGMA busy_timeout = 5000").execute(&pool).await.unwrap();
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed_item_location_position(
        pool: &SqlitePool,
        sku: &str,
        location_id: &str,
        on_hand: i64,
        reserved: i64,
        status: &str,
    ) {
        // item
        sqlx::query(
            r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
               VALUES (?, ?, 'Item EN', '项目', 'part', 'none', ?, ?)"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(sku)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();

        // location
        sqlx::query(
            r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
               VALUES (?, 'WH', 'A', '001', ?)"#,
        )
        .bind(location_id)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();

        // position
        let available = on_hand - reserved;
        sqlx::query(
            r#"INSERT INTO inventory_positions
               (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
               VALUES (?, ?, ?, ?, NULL, ?, ?, ?, ?)"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(sku)
        .bind(location_id)
        .bind(status)
        .bind(on_hand)
        .bind(reserved)
        .bind(available)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_current_position_by_keys() {
        let pool = setup_pool().await;
        let sku = "SKU-001";
        let loc = Uuid::new_v4().to_string();
        seed_item_location_position(&pool, sku, &loc, 100, 30, "available").await;

        let pos = get_current_position(&pool, sku, &loc, "available")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(pos.sku, sku);
        assert_eq!(pos.location_id, loc);
        assert_eq!(pos.status, "available");
        assert_eq!(pos.quantity_on_hand, 100);
        assert_eq!(pos.quantity_reserved, 30);
        assert_eq!(pos.quantity_available, 70);
    }

    #[tokio::test]
    async fn test_ledger_append_only_invariants() {
        let pool = setup_pool().await;
        let loc = Uuid::new_v4().to_string();
        let sku = "SKU-LEDGER";

        // seed minimal item+location
        seed_item_location_position(&pool, sku, &loc, 0, 0, "available").await;

        // insert a movement
        let mv_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, status_before, status_after)
               VALUES (?, datetime('now'), 'user1', 'adjustment', NULL, ?, ?, 10, 'available', 'available')"#,
        )
        .bind(&mv_id)
        .bind(&loc)
        .bind(sku)
        .execute(&pool)
        .await
        .unwrap();

        // Attempt update (should abort per trigger)
        let update_err = sqlx::query("UPDATE inventory_movements SET reason = 'pick' WHERE id = ?")
            .bind(&mv_id)
            .execute(&pool)
            .await
            .unwrap_err();
        let msg = update_err.to_string();
        assert!(msg.contains("append-only") || msg.contains("cannot be updated"));

        // Attempt delete (should abort per trigger)
        let delete_err = sqlx::query("DELETE FROM inventory_movements WHERE id = ?")
            .bind(&mv_id)
            .execute(&pool)
            .await
            .unwrap_err();
        let msg2 = delete_err.to_string();
        assert!(msg2.contains("append-only") || msg2.contains("cannot be deleted"));
    }

    #[tokio::test]
    async fn test_serial_genealogy_and_shipments() {
        let pool = setup_pool().await;
        let sku = "SKU-SERIAL";
        let loc1 = Uuid::new_v4().to_string();
        let loc2 = Uuid::new_v4().to_string();
        let serial = "S12345";

        // Seed item, locations
        sqlx::query(
            r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
               VALUES (?, ?, 'EN', 'ZH', 'part', 'serial', datetime('now'), datetime('now'))"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(sku)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
               VALUES (?, 'WH', 'A', '001', datetime('now'))"#,
        )
        .bind(&loc1)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
               VALUES (?, 'WH', 'B', '002', datetime('now'))"#,
        )
        .bind(&loc2)
        .execute(&pool)
        .await
        .unwrap();

        // Movements: receipt -> transfer -> shipment (explicit timestamps for deterministic order)
        let t1 = Utc::now();
        let t2 = t1 + chrono::Duration::seconds(1);
        let t3 = t1 + chrono::Duration::seconds(2);
        let receipt_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, serial_numbers, status_before, status_after)
               VALUES (?, ?, 'user', 'receipt', NULL, ?, ?, 1, ?, 'available', 'available')"#,
        )
        .bind(&receipt_id)
        .bind(t1.to_rfc3339())
        .bind(&loc1)
        .bind(sku)
        .bind(format!("[\"{}\"]", serial))
        .execute(&pool)
        .await
        .unwrap();

        let transfer_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, serial_numbers, status_before, status_after)
               VALUES (?, ?, 'user', 'transfer', ?, ?, ?, 1, ?, 'available', 'available')"#,
        )
        .bind(&transfer_id)
        .bind(t2.to_rfc3339())
        .bind(&loc1)
        .bind(&loc2)
        .bind(sku)
        .bind(format!("[\"{}\"]", serial))
        .execute(&pool)
        .await
        .unwrap();

        let ship_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, serial_numbers, status_before, status_after)
               VALUES (?, ?, 'user', 'shipment', ?, NULL, ?, -1, ?, 'reserved', 'in_transit')"#,
        )
        .bind(&ship_id)
        .bind(t3.to_rfc3339())
        .bind(&loc2)
        .bind(sku)
        .bind(format!("[\"{}\"]", serial))
        .execute(&pool)
        .await
        .unwrap();

        // Genealogy
        let history = get_serial_genealogy(&pool, serial).await.unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].reason, "receipt");
        assert_eq!(history[1].reason, "transfer");
        assert_eq!(history[2].reason, "shipment");

        // Where shipped
        let shipments = get_serial_shipments(&pool, serial).await.unwrap();
        assert_eq!(shipments.len(), 1);
        assert_eq!(shipments[0].id, ship_id);
    }

    #[tokio::test]
    async fn test_lot_genealogy_and_shipments() {
        let pool = setup_pool().await;
        let sku = "SKU-LOT";
        let loc1 = Uuid::new_v4().to_string();
        let lot_number = format!("LOT-{}", Uuid::new_v4());

        // Seed item, location, lot
        sqlx::query(
            r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
               VALUES (?, ?, 'EN', 'ZH', 'part', 'lot', datetime('now'), datetime('now'))"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(sku)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
               VALUES (?, 'WH', 'A', '001', datetime('now'))"#,
        )
        .bind(&loc1)
        .execute(&pool)
        .await
        .unwrap();
        let lot_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO lots (id, lot_number, sku, expiry_date, created_at)
               VALUES (?, ?, ?, NULL, datetime('now'))"#,
        )
        .bind(&lot_id)
        .bind(&lot_number)
        .bind(sku)
        .execute(&pool)
        .await
        .unwrap();

        // Movements with lot
        let t1 = Utc::now();
        let t2 = t1 + chrono::Duration::seconds(1);
        let receipt_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, lot_id, status_before, status_after)
               VALUES (?, ?, 'user', 'receipt', NULL, ?, ?, 50, ?, 'available', 'available')"#,
        )
        .bind(&receipt_id)
        .bind(t1.to_rfc3339())
        .bind(&loc1)
        .bind(sku)
        .bind(&lot_id)
        .execute(&pool)
        .await
        .unwrap();

        let ship_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO inventory_movements
               (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, lot_id, status_before, status_after)
               VALUES (?, ?, 'user', 'shipment', ?, NULL, ?, -20, ?, 'reserved', 'in_transit')"#,
        )
        .bind(&ship_id)
        .bind(t2.to_rfc3339())
        .bind(&loc1)
        .bind(sku)
        .bind(&lot_id)
        .execute(&pool)
        .await
        .unwrap();

        let history = get_lot_genealogy_by_number(&pool, &lot_number).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].reason, "receipt");
        assert_eq!(history[1].reason, "shipment");

        let shipments = get_lot_shipments_by_number(&pool, &lot_number).await.unwrap();
        assert_eq!(shipments.len(), 1);
        assert_eq!(shipments[0].id, ship_id);
    }
}
